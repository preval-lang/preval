#include <stdio.h>
#include <stdlib.h>
#include <string.h>

__declspec(dllexport) const char *get_message() { return "hello from C!"; }