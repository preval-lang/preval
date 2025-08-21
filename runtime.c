#include <stddef.h>
#include <stdio.h>
struct PrevalString {
  size_t len;
  char *start;
};

void print(struct PrevalString str) {
  printf("%.*s\n", (int)str.len, str.start);
}

// void print() { printf("Hello, World! from C"); }

// int preval_start();

// int main() {
//   int exit = preval_start();
//   printf("Preval exit %d\n", exit);
//   return exit;
// }