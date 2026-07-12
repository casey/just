use super::*;

pub(crate) struct Evaluator<'src: 'run, 'run> {
  assignments: Option<&'run Table<'src, Assignment<'src>>>,
  context: Option<ExecutionContext<'src, 'run>>,
  env: BTreeMap<String, String>,
  is_dependency: bool,
  lists: bool,
  non_const_assignments: HashSet<Number>,
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

  pub(crate) fn evaluate_const_assignments(
    assignments: &'run Table<'src, Assignment<'src>>,
    evaluation_order: &[Name<'src>],
    overrides: &'run HashMap<Number, String>,
    scope: &'run Scope<'src, 'run>,
    variable_references: &HashSet<Number>,
    lists: bool,
  ) -> CompileResult<'src, Self> {
    let mut evaluator = Self {
      assignments: Some(assignments),
      context: None,
      env: BTreeMap::new(),
      is_dependency: false,
      lists,
      non_const_assignments: HashSet::new(),
      overrides,
      recipe: None,
      recursion_depth: 0,
      scope: scope.child(),
    };

    for assignment in evaluation_order {
      let assignment = &assignments[assignment.lexeme()];
      if variable_references.contains(&assignment.number) {
        match evaluator
          .evaluate_assignment(assignment)
          .map_err(Error::unwrap_const)
        {
          Err(ConstEvalError::Const(_)) => {
            evaluator.non_const_assignments.insert(assignment.number);
          }
          Err(error) => return Err(error.into_compile_error()),
          Ok(_) => {}
        }
      }
    }

    Ok(evaluator)
  }

  pub(crate) fn evaluate_sets(
    &mut self,
    sets: Table<'src, Set<'src>>,
  ) -> CompileResult<'src, Settings> {
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
        Setting::DotenvCommand(value) => {
          settings.dotenv_command = self.evaluate_value_const(&value)?;
        }
        Setting::DotenvFilename(value) => {
          settings.dotenv_filename = self.evaluate_value_const(&value)?;
        }
        Setting::DotenvLoad(value) => {
          settings.dotenv_load = value;
        }
        Setting::DotenvPath(value) => {
          settings.dotenv_path = self.evaluate_value_const(&value)?;
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
        Setting::Indentation(_, indentation) => {
          settings.indentation = Some(indentation);
        }
        Setting::Lazy(value) => {
          settings.lazy = value;
        }
        Setting::Lists(value) => {
          settings.lists = value;
        }
        Setting::MinimumVersion(_) => {}
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
          settings.script_interpreter = Some(self.evaluate_interpreter(&value, set.name)?);
        }
        Setting::Shell(value) => {
          settings.shell = Some(self.evaluate_interpreter(&value, set.name)?);
        }
        Setting::Unstable(value) => {
          settings.unstable = value;
        }
        Setting::WindowsPowerShell(value) => {
          settings.windows_powershell = value;
        }
        Setting::WindowsShell(value) => {
          settings.windows_shell = Some(self.evaluate_interpreter(&value, set.name)?);
        }
        Setting::Tempdir(value) => {
          settings.tempdir =
            Some(self.evaluate_string_const(&value, StringContext::Setting(set.name))?);
        }
        Setting::WorkingDirectory(value) => {
          settings.working_directory = Some(
            self
              .evaluate_string_const(&value, StringContext::Setting(set.name))?
              .into(),
          );
        }
      }
    }

    Ok(settings)
  }

  pub(crate) fn evaluate_interpreter(
    &mut self,
    interpreter: &Interpreter<Expression<'src>>,
    setting: Name<'src>,
  ) -> CompileResult<'src, Interpreter<String>> {
    let mut elements = self
      .evaluate_value_const(&interpreter.command)?
      .into_elements();
    for argument in &interpreter.arguments {
      elements.extend(self.evaluate_value_const(argument)?.into_elements());
    }

    let mut elements = elements.into_iter();

    let Some(command) = elements.next() else {
      return Err(ConstEvalError::EmptyInterpreter { setting }.into_compile_error());
    };

    Ok(Interpreter {
      command,
      arguments: elements.collect(),
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
      scope: parent,
      search,
    };

    let mut evaluator = Self {
      assignments: Some(&module.assignments),
      recursion_depth: 0,
      context: Some(context),
      env: BTreeMap::new(),
      is_dependency: false,
      lists: module.settings.lists,
      non_const_assignments: HashSet::new(),
      overrides,
      recipe: None,
      scope: parent.child(),
    };

    for assignment in &module.evaluation_order {
      let assignment = &module.assignments[assignment.lexeme()];
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
    if self.scope.binding(assignment.number).is_none() {
      let value = if let Some(value) = self.overrides.get(&assignment.number) {
        value.into()
      } else {
        self.evaluate_value(&assignment.value)?
      };

      self.scope.bind(Binding {
        attributes: AttributeSet::new(),
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

    Ok(self.scope.value(assignment.number).unwrap())
  }

  fn function_context(&self, name: Name<'src>) -> RunResult<'src, function::Context> {
    Ok(function::Context {
      env: &self.env,
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

    let values = arguments
      .iter()
      .map(|argument| self.evaluate_value(argument))
      .collect::<RunResult<Vec<Value>>>()?;

    let parent = if self.assignments.is_some() {
      &self.scope
    } else {
      context.scope
    };

    let mut scope = parent.child();
    for ((name, number), value) in function.parameters.iter().copied().zip(values) {
      scope.bind(Binding {
        attributes: AttributeSet::new(),
        eager: false,
        export: false,
        file_depth: 0,
        name,
        number,
        prelude: false,
        private: false,
        value,
      });
    }

    let mut evaluator = Evaluator {
      assignments: Some(&context.module.assignments),
      context: Some(context),
      env: BTreeMap::new(),
      is_dependency: self.is_dependency,
      lists: self.lists,
      non_const_assignments: HashSet::new(),
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
    macro_rules! context {
      () => {
        self.function_context(name).unwrap()
      };
    }
    match function {
      Function::Nullary(f) => f(context!()).map(Value::from),
      Function::Unary(f) => {
        let a = self.evaluate_string(&arguments[0], StringContext::Function(name))?;
        f(context!(), &a).map(Value::from)
      }
      Function::UnaryToValue(f) => {
        let a = self.evaluate_string(&arguments[0], StringContext::Function(name))?;
        f(context!(), &a)
      }
      Function::UnaryMap(f) => {
        let a = self.evaluate_value(&arguments[0])?;
        a.elements()
          .iter()
          .map(|element| f(context!(), element))
          .collect()
      }
      Function::UnaryPlus(f) => {
        let a = self.evaluate_string(&arguments[0], StringContext::Function(name))?;
        let mut rest = Vec::new();
        for arg in &arguments[1..] {
          rest.push(self.evaluate_string(arg, StringContext::Function(name))?);
        }
        f(context!(), &a, &rest).map(Value::from)
      }
      Function::BinaryStrValue(f) => {
        let a = self.evaluate_string(&arguments[0], StringContext::Function(name))?;
        let b = self.evaluate_value(&arguments[1])?;
        f(context!(), &a, &b)
      }
      Function::Binary(f) => {
        let a = self.evaluate_string(&arguments[0], StringContext::Function(name))?;
        let b = self.evaluate_string(&arguments[1], StringContext::Function(name))?;
        f(context!(), &a, &b).map(Value::from)
      }
      Function::BinaryToValue(f) => {
        let a = self.evaluate_string(&arguments[0], StringContext::Function(name))?;
        let b = self.evaluate_string(&arguments[1], StringContext::Function(name))?;
        f(context!(), &a, &b)
      }
      Function::BinaryPlus(f) => {
        let a = self.evaluate_string(&arguments[0], StringContext::Function(name))?;
        let b = self.evaluate_string(&arguments[1], StringContext::Function(name))?;
        let mut rest = Vec::new();
        for arg in &arguments[2..] {
          rest.push(self.evaluate_string(arg, StringContext::Function(name))?);
        }
        f(context!(), &a, &b, &rest).map(Value::from)
      }
      Function::Ternary(f) => {
        let a = self.evaluate_string(&arguments[0], StringContext::Function(name))?;
        let b = self.evaluate_string(&arguments[1], StringContext::Function(name))?;
        let c = self.evaluate_string(&arguments[2], StringContext::Function(name))?;
        f(context!(), &a, &b, &c).map(Value::from)
      }
      Function::ValueNullary(f) => f(context!()),
      Function::ValueUnary(f) => {
        let a = self.evaluate_value(&arguments[0])?;
        f(context!(), &a)
      }
      Function::ValueBinary(f) => {
        let a = self.evaluate_value(&arguments[0])?;
        let b = self.evaluate_value(&arguments[1])?;
        f(context!(), &a, &b)
      }
      Function::ValueBinaryOpt(f) => {
        let a = self.evaluate_value(&arguments[0])?;
        let b = if arguments.len() > 1 {
          Some(self.evaluate_value(&arguments[1])?)
        } else {
          None
        };
        f(context!(), &a, b.as_ref())
      }
      Function::BinaryOptToValue(f) => {
        let a = self.evaluate_string(&arguments[0], StringContext::Function(name))?;
        let b = if arguments.len() > 1 {
          Some(self.evaluate_string(&arguments[1], StringContext::Function(name))?)
        } else {
          None
        };
        f(context!(), &a, b.as_deref())
      }
      Function::BinaryOptValueStr(f) => {
        let a = self.evaluate_value(&arguments[0])?;
        let b = if arguments.len() > 1 {
          Some(self.evaluate_string(&arguments[1], StringContext::Function(name))?)
        } else {
          None
        };
        f(context!(), &a, b.as_deref()).map(Value::from)
      }
      Function::BinaryOptValueStrToValue(f) => {
        let a = self.evaluate_value(&arguments[0])?;
        let b = if arguments.len() > 1 {
          Some(self.evaluate_string(&arguments[1], StringContext::Function(name))?)
        } else {
          None
        };
        f(context!(), &a, b.as_deref())
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
    context: StringContext<'src>,
  ) -> RunResult<'src, String> {
    let value = self.evaluate_value(expression)?;

    if value.elements().len() != 1 {
      return Err(Error::ListInStringContext { context, value });
    }

    Ok(value.join())
  }

  pub(crate) fn evaluate_value_const(
    &mut self,
    expression: &Expression<'src>,
  ) -> CompileResult<'src, Value> {
    assert!(self.context.is_none());
    self
      .evaluate_value(expression)
      .map_err(|error| error.unwrap_const().into_compile_error())
  }

  pub(crate) fn evaluate_string_const(
    &mut self,
    expression: &Expression<'src>,
    context: StringContext<'src>,
  ) -> CompileResult<'src, String> {
    assert!(self.context.is_none());
    self
      .evaluate_string(expression, context)
      .map_err(|error| error.unwrap_const().into_compile_error())
  }

  pub(crate) fn evaluate_value(&mut self, expression: &Expression<'src>) -> RunResult<'src, Value> {
    match expression {
      Expression::And { lhs, rhs } => {
        let lhs = self.evaluate_value(lhs)?;
        if lhs.is_truthy() {
          self.evaluate_value(rhs)
        } else {
          Ok(Value::new())
        }
      }
      Expression::Assert {
        condition,
        message,
        name,
      } => {
        let value = self.evaluate_value(condition)?;
        if value.is_truthy() {
          Ok(if self.lists { value } else { Value::from("") })
        } else {
          Err(Error::Assert {
            message: if let Some(message) = message {
              self.evaluate_value(message)?.join()
            } else {
              format!("`{condition}`")
            },
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
      Expression::Comparison { .. } => Ok(self.evaluate_boolean(expression)?.into()),
      Expression::Concatenation { lhs, operator, rhs } => {
        let lhs = self.evaluate_value(lhs)?;
        let rhs = self.evaluate_value(rhs)?;
        lhs.apply(&rhs, ListOperator::Concatenate, *operator)
      }
      Expression::ListConcatenation { lhs, rhs, .. } => {
        let lhs = self.evaluate_value(lhs)?;
        let rhs = self.evaluate_value(rhs)?;
        Ok(
          lhs
            .into_elements()
            .into_iter()
            .chain(rhs.into_elements())
            .collect(),
        )
      }
      Expression::Conditional {
        condition,
        then,
        otherwise,
      } => {
        if self.evaluate_boolean(condition)? {
          self.evaluate_value(then)
        } else if let Some(otherwise) = otherwise {
          self.evaluate_value(otherwise)
        } else {
          Ok(Value::new())
        }
      }
      Expression::FormatString { start, expressions } => {
        let mut value = start.cooked.clone();

        for (expression, string) in expressions {
          value.push_str(&self.evaluate_value(expression)?.join());
          value.push_str(&string.cooked);
        }

        if start.kind.indented {
          Ok(unindent(&value).into())
        } else {
          Ok(value.into())
        }
      }
      Expression::Group { contents } => self.evaluate_value(contents),
      Expression::Join {
        lhs: None,
        operator,
        rhs,
      } => {
        let rhs = self.evaluate_value(rhs)?;
        Value::from("").apply(&rhs, ListOperator::Join, *operator)
      }
      Expression::Join {
        lhs: Some(lhs),
        operator,
        rhs,
      } => {
        let lhs = self.evaluate_value(lhs)?;
        let rhs = self.evaluate_value(rhs)?;
        lhs.apply(&rhs, ListOperator::Join, *operator)
      }
      Expression::List { elements, .. } => {
        let mut values = Vec::new();
        for element in elements {
          values.extend(self.evaluate_value(element)?.into_elements());
        }
        Ok(values.into())
      }
      Expression::Not { operand } => Ok((!self.evaluate_value(operand)?.is_truthy()).into()),
      Expression::Or { lhs, rhs } => {
        let lhs = self.evaluate_value(lhs)?;
        if lhs.is_truthy() {
          Ok(lhs)
        } else {
          self.evaluate_value(rhs)
        }
      }
      Expression::StringLiteral { string_literal } => Ok(string_literal.cooked.deref().into()),
      Expression::Variable { name, number } => {
        let Some(number) = number else {
          return Err(Error::internal(format!(
            "attempted to evaluate unresolved variable `{name}`"
          )));
        };

        if self.non_const_assignments.contains(number) {
          Err(ConstError::Variable(*name).into())
        } else if let Some(binding) = self.scope.binding(*number) {
          Ok(binding.value.clone())
        } else if let Some(assignment) = self
          .assignments
          .and_then(|assignments| assignments.assignment(*number))
        {
          Ok(self.evaluate_assignment(assignment)?.clone())
        } else {
          Err(Error::internal(format!(
            "attempted to evaluate undefined variable `{name}`"
          )))
        }
      }
    }
  }

  fn evaluate_boolean(&mut self, condition: &Expression<'src>) -> RunResult<'src, bool> {
    let Expression::Comparison {
      lhs,
      operator,
      rhs,
      token,
    } = condition
    else {
      return Ok(self.evaluate_value(condition)?.is_truthy());
    };
    let condition = match operator {
      ConditionalOperator::Equality => self.evaluate_value(lhs)? == self.evaluate_value(rhs)?,
      ConditionalOperator::Inequality => self.evaluate_value(lhs)? != self.evaluate_value(rhs)?,
      ConditionalOperator::RegexMatch | ConditionalOperator::RegexMismatch => {
        let lhs = self.evaluate_value(lhs)?;
        let rhs = self.evaluate_value(rhs)?;

        let regexes = rhs
          .elements()
          .iter()
          .map(|regex| Regex::new(regex))
          .collect::<Result<Vec<Regex>, regex::Error>>()
          .map_err(|source| Error::RegexCompile {
            source,
            token: *token,
          })?;

        let matched = lhs
          .elements()
          .iter()
          .any(|element| regexes.iter().any(|regex| regex.is_match(element)));

        match operator {
          ConditionalOperator::RegexMatch => matched,
          ConditionalOperator::RegexMismatch => !matched,
          _ => unreachable!(),
        }
      }
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
    assert!(!context.config.dry_run);

    let mut cmd = context.module.settings.shell_command(context.config);

    cmd.arg(command);

    if let Some(args) = args {
      if ShellKind::from(&cmd).takes_shell_name() {
        cmd.arg(command);
      }

      cmd.args(args);
    }

    let environment = Environment::new(
      context.dotenv,
      scope,
      &context.module.settings,
      &context.module.unexports,
    );

    environment.export(&mut cmd);

    cmd
      .current_dir(context.working_directory())
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
          evaluated += &self.evaluate_value(expression)?.join();
        }
      }
    }
    Ok(evaluated)
  }

  pub(crate) fn evaluate_parameters(
    arguments: &[Value],
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
        let context = StringContext::EnvKey(recipe.attributes.name(attribute));
        let key = evaluator.evaluate_string(key, context)?;
        let value = evaluator.evaluate_value(value)?;
        if !value.is_empty() {
          evaluator.env.insert(key, value.join());
        }
      }
    }

    let mut positional = Vec::new();

    if arguments.len() != parameters.len() {
      return Err(Error::internal("arguments do not match parameter count"));
    }

    for (parameter, argument) in parameters.iter().zip(arguments) {
      let value = if argument.elements().is_empty() {
        if let Some(default) = &parameter.default {
          evaluator.evaluate_value(default)?
        } else if parameter.kind == ParameterKind::Star || parameter.flag {
          Value::new()
        } else {
          return Err(Error::EmptyListArgument {
            parameter: parameter.name.lexeme(),
            recipe: recipe.name(),
          });
        }
      } else if let Some(value) = &parameter.value
        && !evaluator.is_dependency
      {
        iter::repeat_n(
          evaluator.evaluate_value(value)?.elements(),
          argument.elements().len(),
        )
        .flatten()
        .cloned()
        .collect()
      } else {
        argument.clone()
      };

      parameter.check_value_count(recipe, &value)?;

      for element in &value {
        parameter.check_pattern_match(recipe, element)?;
      }

      if parameter.kind.is_variadic() || parameter.multiple {
        positional.extend(value.elements().iter().cloned());
      } else {
        positional.push(value.join());
      }

      let value = if evaluator.lists {
        value
      } else {
        value.join().into()
      };

      evaluator.scope.bind(Binding {
        attributes: AttributeSet::new(),
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
      lists: context.module.settings.lists,
      non_const_assignments: HashSet::new(),
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
