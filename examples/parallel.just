# GNU parallel can be used to run tasks concurrently

parallel:
  #!/usr/bin/env -S parallel --shebang --ungroup --jobs {{ num_cpus() }}
  echo task 1 start; sleep 3; echo task 1 done
  echo task 2 start; sleep 3; echo task 2 done
  echo task 3 start; sleep 3; echo task 3 done
  echo task 4 start; sleep 3; echo task 4 done
