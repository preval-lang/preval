#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include "preval.h"

__declspec(dllexport) Value* print(API* api, size_t argc, const Value * const * args) {
    if (!args[0] || !args[1]) {
        return NULL;
    }

    printf("%.*s\n", (int)api->string_value_length(args[1]), api->string_value_start(args[1]));
    return api->new_tuple_value();
}

__declspec(dllexport) Value* input(API* api, size_t argc, const Value * const * args) {
    if (!args[0]) {
        return NULL;
    }

    uint8_t* line = NULL;
    size_t len = 0;

    uint8_t *buffer = NULL;
    size_t size = 0;
    int c;
    while ((c = getchar()) != '\n' && c != EOF) {
            uint8_t *tmp = realloc(buffer, size + 2); // +1 char +1 '\0'
            if (!tmp) {
                free(buffer);
                printf("ERROR! TODO: make a way for errors to be propagated to preval");
                exit(1);
            }
            buffer = tmp;
            buffer[size++] = c;
        }

    Value* result = api->new_string_value(buffer, size);

    if (buffer) {
        free(buffer);
    }

    return result;
}
