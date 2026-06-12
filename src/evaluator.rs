use super::*;

pub(crate) struct Evaluator<'src: 'run, 'run> {
  assignments: Option<&'run Table<'src, Assignment<'src>>>,
  context: Option<ExecutionContext<'src, 'run>>,
  env: BTreeMap<String, String>,
  is_dependency: bool,
  non_const_assignments: Table<'src, Name<'src>>,
  overrides: &'run HashMap<Number, String>,
  recipe: Option<Name<'src>>,
  recursion_depth: usize,
  scope: Scope<'src, 'run>,
}

impl<'src, 'run> Evaluator<'src, 'run> {
  fn context(
    &self,
    const_error: ConstError<'src>,
  ) -> Result<&ExecutionContext<'src, 'run>, ConstError<'src>> {
    self.context.as_ref().ok_or(const_error)
  }

  pub(crate) fn evaluate_settings(
    assignments: &'run Table<'src, Assignment<'src>>,
    overrides: &'run HashMap<Number, String>,
    scope: &'run Scope<'src, 'run>,
    sets: Table<'src, Set<'src>>,
  ) -> RunResult<'src, Settings> {
    let mut evaluator = Self {
      assignments: Some(assignments),
      recursion_depth: 0,
      context: None,
      env: BTreeMap::new(),
      is_dependency: false,
      non_const_assignments: Table::new(),
      overrides,
      recipe: None,
      scope: scope.child(),
    };

    let variable_references = sets
      .values()
      .flat_map(|set| set.value.expressions())
      .flat_map(|expression| expression.references())
      .filter_map(|reference| {
        if let Reference::Variable(variable) = reference {
          Some(variable.lexeme())
        } else {
          None
        }
      })
      .collect::<BTreeSet<&str>>();

    for assignment in assignments.values() {
      if variable_references.contains(assignment.name.lexeme()) {
        match evaluator.evaluate_assignment(assignment) {
          Err(Error::Const { .. }) => evaluator.non_const_assignments.insert(assignment.name),
          Err(err) => return Err(err),
          Ok(_) => {}
        }
      }
    }

    evaluator.evaluate_sets(sets)
  }

  fn evaluate_sets(&mut self, sets: Table<'src, Set<'src>>) -> RunResult<'src, Settings> {
    let mut settings = Settings::default();

    for (_name, set) in sets {
      match set.value {
        Setting::AllowDuplicateRecipes(value) => {
          settings.allow_duplicate_recipes = value;
        }
        Setting::AllowDuplicateVariables(value) => {
          settings.allow_duplicate_variables = value;
        }
        Setting::DefaultList(value) => {
          settings.default_list = value;
        }
        Setting::DefaultScript(value) => {
          settings.default_script = value;
        }
        Setting::DotenvFilename(value) => {
          settings.dotenv_filename = Some(self.evaluate_string(&value)?);
        }
        Setting::DotenvLoad(value) => {
          settings.dotenv_load = value;
        }
        Setting::DotenvPath(value) => {
          settings.dotenv_path = Some(self.evaluate_string(&value)?.into());
        }
        Setting::DotenvOverride(value) => {
          settings.dotenv_override = value;
        }
        Setting::DotenvRequired(value) => {
          settings.dotenv_required = value;
        }
        Setting::Export(value) => {
          settings.export = value;
        }
        Setting::Fallback(value) => {
          settings.fallback = value;
        }
        Setting::Guards(guards) => {
          settings.guards = guards;
        }
        Setting::IgnoreComments(value) => {
          settings.ignore_comments = value;
        }
        Setting::Lazy(value) => {
          settings.lazy = value;
        }
        Setting::Lists(value) => {
          settings.lists = value;
        }
        Setting::NoCd(value) => {
          settings.no_cd = value;
        }
        Setting::NoExitMessage(value) => {
          settings.no_exit_message = value;
        }
        Setting::PositionalArguments(value) => {
          settings.positional_arguments = value;
        }
        Setting::Quiet(value) => {
          settings.quiet = value;
        }
        Setting::ScriptInterpreter(value) => {
          settings.script_interpreter = Some(self.evaluate_interpreter(&value)?);
        }
        Setting::Shell(value) => {
          settings.shell = Some(self.evaluate_interpreter(&value)?);
        }
        Setting::Unstable(value) => {
          settings.unstable = value;
        }
        Setting::WindowsPowerShell(value) => {
          settings.windows_powershell = value;
        }
        Setting::WindowsShell(value) => {
          settings.windows_shell = Some(self.evaluate_interpreter(&value)?);
        }
        Setting::Tempdir(value) => {
          settings.tempdir = Some(self.evaluate_string(&value)?);
        }
        Setting::WorkingDirectory(value) => {
          settings.working_directory = Some(self.evaluate_string(&value)?.into());
        }
      }
    }

    Ok(settings)
  }

  pub(crate) fn evaluate_interpreter(
    &mut self,
    interpreter: &Interpreter<Expression<'src>>,
  ) -> RunResult<'src, Interpreter<String>> {
    Ok(Interpreter {
      command: self.evaluate_string(&interpreter.command)?,
      arguments: interpreter
        .arguments
        .iter()
        .map(|argument| self.evaluate_string(argument))
        .collect::<RunResult<Vec<String>>>()?,
    })
  }

  pub(crate) fn evaluate_assignments(
    config: &'run Config,
    dotenv: &'run BTreeMap<String, String>,
    module: &'run Justfile<'src>,
    overrides: &'run HashMap<Number, String>,
    parent: &'run Scope<'src, 'run>,
    search: &'run Search,
    variable_references: Option<&HashSet<Number>>,
  ) -> RunResult<'src, Scope<'src, 'run>>
  where
    'src: 'run,
  {
    let context = ExecutionContext {
      config,
      dotenv,
      module,
      overrides,
      search,
    };

    let mut evaluator = Self {
      assignments: Some(&module.assignments),
      recursion_depth: 0,
      context: Some(context),
      env: BTreeMap::new(),
      is_dependency: false,
      non_const_assignments: Table::new(),
      overrides,
      recipe: None,
      scope: parent.child(),
    };

    for assignment in module.assignments.values() {
      if assignment.eager
        || assignment.export
        || module.settings.export
        || variable_references
          .is_none_or(|variable_references| variable_references.contains(&assignment.number))
      {
        evaluator.evaluate_assignment(assignment)?;
      }
    }

    Ok(evaluator.scope)
  }

  fn evaluate_assignment(&mut self, assignment: &Assignment<'src>) -> RunResult<'src, &Value> {
    let name = assignment.name.lexeme();

    if !self.scope.bound(name) {
      let value = if let Some(value) = self.overrides.get(&assignment.number) {
        value.into()
      } else {
        self.evaluate_value(&assignment.value)?
      };

      self.scope.bind(Binding {
        eager: assignment.eager,
        export: assignment.export
          || self
            .context
            .is_some_and(|context| context.module.settings.export),
        file_depth: 0,
        name: assignment.name,
        number: assignment.number,
        prelude: false,
        private: assignment.private,
        value,
      });
    }

    Ok(self.scope.value(name).unwrap())
  }

  fn function_context(&self, name: Name<'src>) -> RunResult<'src, function::Context> {
    Ok(function::Context {
      execution_context: self.context(ConstError::FunctionCall(name))?,
      is_dependency: self.is_dependency,
      name,
      recipe: self.recipe,
      scope: &self.scope,
    })
  }

  fn evaluate_defined_function(
    &mut self,
    function: &FunctionDefinition<'src>,
    arguments: &[Expression<'src>],
  ) -> RunResult<'src, Value> {
    let recursion_depth = self.recursion_depth + 1;

    if recursion_depth == RECURSION_LIMIT {
      return Err(Error::RecursionLimit {
        last: function.name,
      });
    }

    let context = *self.context.as_ref().unwrap();

    let mut scope = Scope::root();
    for ((name, number), argument) in function.parameters.iter().copied().zip(arguments) {
      let value = self.evaluate_value(argument)?;
      scope.bind(Binding {
        eager: false,
        export: false,
        file_depth: 0,
        name,
        number,
        prelude: false,
        private: false,
        value: value.clone(),
      });
    }

    let mut evaluator = Evaluator {
      assignments: Some(&context.module.assignments),
      context: Some(context),
      env: BTreeMap::new(),
      is_dependency: self.is_dependency,
      non_const_assignments: Table::new(),
      overrides: self.overrides,
      recipe: self.recipe,
      recursion_depth,
      scope,
    };

    evaluator.evaluate_value(&function.body)
  }

  fn evaluate_builtin_function(
    &mut self,
    name: Name<'src>,
    function: Function,
    arguments: &[Expression<'src>],
  ) -> RunResult<'src, Value> {
    match function {
      Function::Nullary(f) => f(self.function_context(name).unwrap()).map(Value::from),
      Function::Unary(f) => {
        let a = self.evaluate_string(&arguments[0])?;
        f(self.function_context(name).unwrap(), &a).map(Value::from)
      }
      Function::UnaryList(f) => {
        let a = self.evaluate_value(&arguments[0])?;
        f(self.function_context(name).unwrap(), &a)
      }
      Function::UnaryOpt(f) => {
        let a = self.evaluate_string(&arguments[0])?;
        let b = if arguments.len() > 1 {
          Some(self.evaluate_string(&arguments[1])?)
        } else {
          None
        };
        f(self.function_context(name).unwrap(), &a, b.as_deref()).map(Value::from)
      }
      Function::UnaryPlus(f) => {
        let a = self.evaluate_string(&arguments[0])?;
        let mut rest = Vec::new();
        for arg in &arguments[1..] {
          rest.push(self.evaluate_string(arg)?);
        }
        f(self.function_context(name).unwrap(), &a, &rest).map(Value::from)
      }
      Function::Binary(f) => {
        let a = self.evaluate_string(&arguments[0])?;
        let b = self.evaluate_string(&arguments[1])?;
        f(self.function_context(name).unwrap(), &a, &b).map(Value::from)
      }
      Function::BinaryPlus(f) => {
        let a = self.evaluate_string(&arguments[0])?;
        let b = self.evaluate_string(&arguments[1])?;
        let mut rest = Vec::new();
        for arg in &arguments[2..] {
          rest.push(self.evaluate_string(arg)?);
        }
        f(self.function_context(name).unwrap(), &a, &b, &rest).map(Value::from)
      }
      Function::Ternary(f) => {
        let a = self.evaluate_string(&arguments[0])?;
        let b = self.evaluate_string(&arguments[1])?;
        let c = self.evaluate_string(&arguments[2])?;
        f(self.function_context(name).unwrap(), &a, &b, &c).map(Value::from)
      }
    }
    .map_err(|message| Error::FunctionCall {
      function: name,
      message,
    })
  }

  pub(crate) fn evaluate_string(
    &mut self,
    expression: &Expression<'src>,
  ) -> RunResult<'src, String> {
    Ok(self.evaluate_value(expression)?.into_string())
  }

  pub(crate) fn evaluate_value(&mut self, expression: &Expression<'src>) -> RunResult<'src, Value> {
    match expression {
      Expression::And { lhs, rhs } => {
        let lhs = self.evaluate_value(lhs)?;
        if lhs.is_empty() {
          return Ok(Value::from(""));
        }
        self.evaluate_value(rhs)
      }
      Expression::Assert {
        condition,
        error,
        name,
      } => {
        if self.evaluate_condition(condition)? {
          Ok(Value::from(""))
        } else {
          Err(Error::Assert {
            message: self.evaluate_string(error)?,
            name: *name,
          })
        }
      }
      Expression::Backtick { contents, token } => {
        let context = self.context(ConstError::Backtick(*token))?;

        if context.config.dry_run {
          return Ok(Value::from(format!("`{contents}`")));
        }

        Self::run_command(context, &self.env, &self.scope, contents, None)
          .map(Value::from)
          .map_err(|output_error| Error::Backtick {
            token: *token,
            output_error,
          })
      }
      Expression::Call { name, arguments } => {
        let module = self.context(ConstError::FunctionCall(*name))?.module;
        if let Some(function) = module.functions.get(name.lexeme()) {
          self.evaluate_defined_function(function, arguments)
        } else if let Some(builtin) = function::get(name.lexeme()) {
          self.evaluate_builtin_function(*name, builtin, arguments)
        } else {
          unreachable!();
        }
      }
      Expression::Concatenation { lhs, rhs } => {
        let lhs = self.evaluate_string(lhs)?;
        let rhs = self.evaluate_string(rhs)?;
        Ok((lhs + &rhs).into())
      }
      Expression::Conditional {
        condition,
        then,
        otherwise,
      } => {
        if self.evaluate_condition(condition)? {
          self.evaluate_value(then)
        } else {
          self.evaluate_value(otherwise)
        }
      }
      Expression::FormatString { start, expressions } => {
        let mut value = start.cooked.clone();

        for (expression, string) in expressions {
          value.push_str(&self.evaluate_string(expression)?);
          value.push_str(&string.cooked);
        }

        if start.kind.indented {
          Ok(unindent(&value).into())
        } else {
          Ok(value.into())
        }
      }
      Expression::Group { contents } => self.evaluate_value(contents),
      Expression::Join { lhs: None, rhs } => {
        Ok(("/".to_string() + &self.evaluate_string(rhs)?).into())
      }
      Expression::Join {
        lhs: Some(lhs),
        rhs,
      } => {
        let lhs = self.evaluate_string(lhs)?;
        let rhs = self.evaluate_string(rhs)?;
        Ok((lhs + "/" + &rhs).into())
      }
      Expression::Or { lhs, rhs } => {
        let lhs = self.evaluate_value(lhs)?;
        if !lhs.is_empty() {
          return Ok(lhs);
        }
        self.evaluate_value(rhs)
      }
      Expression::StringLiteral { string_literal } => Ok(string_literal.cooked.deref().into()),
      Expression::Variable { name, .. } => {
        let variable = name.lexeme();
        if let Some(value) = self.scope.value(variable) {
          Ok(value.clone())
        } else if self.non_const_assignments.contains_key(name.lexeme()) {
          Err(ConstError::Variable(*name).into())
        } else if let Some(assignment) = self
          .assignments
          .and_then(|assignments| assignments.get(variable))
        {
          Ok(self.evaluate_assignment(assignment)?.clone())
        } else {
          Err(Error::internal(format!(
            "attempted to evaluate undefined variable `{variable}`"
          )))
        }
      }
    }
  }

  fn evaluate_condition(&mut self, condition: &Condition<'src>) -> RunResult<'src, bool> {
    let lhs_value = self.evaluate_string(&condition.lhs)?;
    let rhs_value = self.evaluate_string(&condition.rhs)?;
    let condition = match condition.operator {
      ConditionalOperator::Equality => lhs_value == rhs_value,
      ConditionalOperator::Inequality => lhs_value != rhs_value,
      ConditionalOperator::RegexMatch => Regex::new(&rhs_value)
        .map_err(|source| Error::RegexCompile { source })?
        .is_match(&lhs_value),
      ConditionalOperator::RegexMismatch => !Regex::new(&rhs_value)
        .map_err(|source| Error::RegexCompile { source })?
        .is_match(&lhs_value),
    };
    Ok(condition)
  }

  pub(crate) fn run_command(
    context: &ExecutionContext,
    env: &BTreeMap<String, String>,
    scope: &Scope,
    command: &str,
    args: Option<&[String]>,
  ) -> Result<String, OutputError> {
    let mut cmd = context.module.settings.shell_command(context.config);

    cmd.arg(command);

    if let Some(args) = args {
      if ShellKind::from(&cmd).takes_shell_name() {
        cmd.arg(command);
      }

      cmd.args(args);
    }

    cmd
      .current_dir(context.working_directory())
      .export(
        &context.module.settings,
        context.dotenv,
        scope,
        &context.module.unexports,
      )
      .stdin(Stdio::inherit())
      .stderr(if context.config.verbosity.quiet() {
        Stdio::null()
      } else {
        Stdio::inherit()
      })
      .stdout(Stdio::piped());

    for (key, value) in env {
      cmd.env(key, value);
    }

    cmd.output_guard_stdout()
  }

  pub(crate) fn evaluate_line(
    &mut self,
    line: &Line<'src>,
    continued: bool,
  ) -> RunResult<'src, String> {
    let mut evaluated = String::new();
    for (i, fragment) in line.fragments.iter().enumerate() {
      match fragment {
        Fragment::Text { token } => {
          let lexeme = token
            .lexeme()
            .replace(Lexer::INTERPOLATION_ESCAPE, Lexer::INTERPOLATION_START);

          if i == 0 && continued {
            evaluated += lexeme.trim_start();
          } else {
            evaluated += &lexeme;
          }
        }
        Fragment::Interpolation { expression } => {
          evaluated += &self.evaluate_string(expression)?;
        }
      }
    }
    Ok(evaluated)
  }

  pub(crate) fn evaluate_parameters(
    arguments: &[Vec<String>],
    context: &ExecutionContext<'src, 'run>,
    is_dependency: bool,
    parameters: &[Parameter<'src>],
    recipe: &Recipe<'src>,
    scope: &'run Scope<'src, 'run>,
  ) -> RunResult<'src, (Scope<'src, 'run>, Vec<String>, BTreeMap<String, String>)> {
    let mut evaluator = Self::new(
      context,
      BTreeMap::new(),
      is_dependency,
      Some(recipe.name),
      scope,
    );

    for attribute in &recipe.attributes {
      if let Attribute::Env(key, value) = attribute {
        let key = evaluator.evaluate_string(key)?;
        let value = evaluator.evaluate_string(value)?;
        evaluator.env.insert(key, value);
      }
    }

    let mut positional = Vec::new();

    if arguments.len() != parameters.len() {
      return Err(Error::internal("arguments do not match parameter count"));
    }

    for (parameter, group) in parameters.iter().zip(arguments) {
      let value = if group.is_empty() {
        if let Some(ref default) = parameter.default {
          let value = evaluator.evaluate_value(default)?;
          positional.push(value.join().into_owned());
          value
        } else if parameter.kind == ParameterKind::Star {
          Value::new()
        } else {
          return Err(Error::internal("missing parameter without default"));
        }
      } else if parameter.kind.is_variadic() {
        positional.extend_from_slice(group);
        group.iter().cloned().collect()
      } else {
        if group.len() != 1 {
          return Err(Error::internal(
            "multiple values for non-variadic parameter",
          ));
        }
        positional.push(group[0].clone());
        Value::from(group[0].clone())
      };

      for element in value.elements() {
        parameter.check_pattern_match(recipe, element)?;
      }

      let value = if context.module.settings.lists {
        value
      } else {
        value.into_string().into()
      };

      evaluator.scope.bind(Binding {
        eager: false,
        export: parameter.export,
        file_depth: 0,
        name: parameter.name,
        number: parameter.number,
        prelude: false,
        private: false,
        value,
      });
    }

    Ok((evaluator.scope, positional, evaluator.env))
  }

  pub(crate) fn new(
    context: &ExecutionContext<'src, 'run>,
    env: BTreeMap<String, String>,
    is_dependency: bool,
    recipe: Option<Name<'src>>,
    scope: &'run Scope<'src, 'run>,
  ) -> Self {
    Self {
      assignments: None,
      context: Some(*context),
      env,
      is_dependency,
      non_const_assignments: Table::new(),
      overrides: context.overrides,
      recipe,
      recursion_depth: 0,
      scope: scope.child(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  run_error! {
    name: backtick_code,
    src: "
      a:
       echo {{`f() { return 100; }; f`}}
    ",
    args: ["a"],
    error: Error::Backtick {
      token,
      output_error: OutputError::Code(code),
    },
    check: {
      assert_eq!(code, 100);
      assert_eq!(token.lexeme(), "`f() { return 100; }; f`");
    }
  }

  run_error! {
    name: export_assignment_backtick,
    src: "
      export exported_variable := 'A'
      b := `echo $exported_variable`

      recipe:
        echo {{b}}
    ",
    args: ["--quiet", "recipe"],
    error: Error::Backtick {
        token,
        output_error: OutputError::Code(_),
    },
    check: {
      assert_eq!(token.lexeme(), "`echo $exported_variable`");
    }
  }
}
