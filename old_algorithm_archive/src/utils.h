#ifndef UTILS_H 
#define UTILS_H 

#include "common.h"


/*functions and macros*/

extern void er_report( FILE * flux, const char * repport_msg, const char * error_msg , err_code flag);
/*
    flux -> not null 
    repport_msg -> not null 
    error_msg -> not null 
    flag -> 

    reports an error in the flux given. 

    Some wrappers are defined around this function to avoid code redundancy
*/

//some wrappers arround the er_report function ; I recommand sticking to the def_err/war_handlers 
//bc passing a code block to a macro is very much not safe lol
#define error_handler(cond, msg,flag,handler_code) if((cond)){er_report(stderr, "error", (msg), (flag)); {handler_code;} return (flag);}
/*
    the error_handler macro function checks if cond==TRUE, reports the error 
    associated with flag if it's the case and executes hanlder_code before returning the value of flag.
*/

#define warning_handler(cond,msg,flag, handler_code) if((cond)){er_report(stderr, "warning", (msg), (flag)); {handler_code;}}
/*
same as error_hanler but doesn't return anything
*/
//I really like these macros I just think they're neat the C preprocessor is crazy for this ngl 
/*
    the default macro functions are safer / easier to use. I recommand sticking to them. 
    they don't execute any code and just report the error / warning if cond is true.
*/
#define def_err_handler(cond,msg,flag) error_handler(cond,msg,flag,;)
#define def_war_handler(cond,msg,flag) warning_handler(cond,msg,flag,;)


#define DEC_BASE 10
#define BUFF_SIZE 65536

extern void skip_char(char chr , char** buff);
/*
    sets *buff to it's first char that isn't chr.
*/
extern err_code fill_arr(char* str_source, uint32_t* arr_dest, uint32_t arr_size);
/*
    str_source & arr_dest -> not null 

    parses number from str source and writes them in arr_dest while str_source contains
    number and the number of elements written in arr_dest is less than arr_size
*/

extern FILE* fopen_check(const char* filename, const char* mode);
/*
    some wrappers around fopen
    to check for errors
*/

#endif
