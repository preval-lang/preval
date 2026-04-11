#include <stdio.h>
#include "preval.h"

__declspec(dllexport) Value* print(API* api, size_t argc, const Value * const * args) {
    if (!args[0] || !args[1]) {
        return NULL;
    }

    printf("%.*s\n", (int)api->string_value_length(args[1]), api->string_value_start(args[1]));
    Value* tuple = api->new_tuple_value();
    return tuple;
}
