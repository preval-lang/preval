#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct Value Value;

typedef struct API {
  void (*drop_value)(struct Value*);
  uintptr_t (*string_value_length)(const struct Value*);
  const uint8_t *(*string_value_start)(const struct Value*);
  struct Value *(*new_tuple_value)(void);
} API;
