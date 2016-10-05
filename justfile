test:
	cargo test
	cargo run -- quine clean > /dev/null 2> /dev/null

# list all recipies
list:
	@$(MAKE) -pRrq -f $(lastword $(MAKEFILE_LIST)) : 2>/dev/null | awk -v RS= -F: '/^# File/,/^# Finished Make data base/ {if ($$1 !~ "^[#.]") {print $$1}}' | sort | egrep -v -e '^[^[:alnum:]]' -e '^$@$$' | xargs

args:
	@echo "I got some arguments: ARG0=${ARG0} ARG1=${ARG1} ARG2=${ARG2}"

# make a quine, compile it, and verify it
quine: create
	cc tmp/gen0.c -o tmp/gen0
	./tmp/gen0 > tmp/gen1.c
	cc tmp/gen1.c -o tmp/gen1
	./tmp/gen1 > tmp/gen2.c
	diff tmp/gen1.c tmp/gen2.c
	@echo 'It was a quine!'

# create our quine
create:
	mkdir -p tmp
	echo 'int printf(const char*, ...); int main() { char *s = "int printf(const char*, ...); int main() { char *s = %c%s%c; printf(s, 34, s, 34); return 0; }"; printf(s, 34, s, 34); return 0; }' > tmp/gen0.c


# clean up
clean:
	rm -r tmp
