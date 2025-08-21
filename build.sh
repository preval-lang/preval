gcc runtime.c -c -o runtime.o -ggdb -O0
as out.s -o out.o -ggdb
gcc -no-pie out.o runtime.o -o out -ggdb
