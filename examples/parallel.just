# This is an example of using GNU parallel with Just
# to run concurrent tasks

# To accumulate output from each job and print it after job is done,
# remove --ungroup switch.

churn:
  #!/usr/bin/env -S parallel --shebang --ungroup --jobs {{ num_cpus() }}
  echo task 1 start; sleep 3; echo task 1 done
  echo task 2 start; sleep 3; echo task 2 done
  echo task 3 start; sleep 3; echo task 3 done
  echo task 4 start; echo task 4 done
