set unstable

head(s) := if s =~ '^0' {
  '0'
} else if s =~ '^1' {
  '1'
} else {
  '0'
}

tail(s) := if s =~ '^0' {
  trim_start_match(s, '0')
} else if s =~ '^1' {
  trim_start_match(s, '1')
} else {
  ''
}

rule(l, c, r) := if l + c + r == '111' {
  '0'
} else if l + c + r == '110' {
  '1'
} else if l + c + r == '101' {
  '1'
} else if l + c + r == '100' {
  '0'
} else if l + c + r == '011' {
  '1'
} else if l + c + r == '010' {
  '1'
} else if l + c + r == '001' {
  '1'
} else {
  '0'
}

step(l, rest, acc) := if rest == '' {
  acc
} else {
  step(head(rest), tail(rest), acc + rule(l, head(rest), head(tail(rest))))
}

evolve(cells) := step('0', cells, '')

pixel(c) := if c == '1' { '█' } else { ' ' }

show(s) := if s == '' { '' } else { pixel(head(s)) + show(tail(s)) }

dec(n) := trim_start_match(n, '.')

run(cells, n) := if n == '' {
  show(cells)
} else {
  show(cells) + "\n" + run(evolve(cells), dec(n))
}

initial := '0000000000000000000000000000000000000000000000000000000000001'

steps := '..............................'

default:
  @echo '{{ run(initial, steps) }}'
