#ifndef COMMON_H
#define COMMON_H

/*



	common.h defines handy includes and some macros
*/

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

typedef uint8_t err_code;

enum Errors {
	ErrOk = 0,
	ErrNull,
	ErrAlloc,
	ErrVal,
	ErrFormat,
};

#define debug

#endif
