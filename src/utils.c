#include "utils.h"
#include <stdio.h>

static inline char * str_flag(err_code flag) {
    switch (flag) { 
        case ERR_NULL: return "null pointer passed";
        case ERR_ALLOC: return "couldn't allocate memory";
        case ERR_REALLOC: return "couldn't reallocate memory";
        case ERR_VAL: return "value given not matching expected values";
        default : return "unknown error";
    }
}//ok

void er_report( FILE * flux, const char * repport_msg, const char * error_msg ,err_code flag)
{
        /*
        flux -> not null 
        repport_msg -> not null 
        error_msg -> not null 
        flag -> 
        the real repport function of the project 
        every variant of it will rely on it 
        */
        if(! flux){
                fprintf(stderr,"er_report : NULL was passed for flux\n");
            
        }else if(!repport_msg){
                fprintf(stderr,"er_report : NULL was passed for repport_msg\n");
             
        }else if(!error_msg){
                fprintf(stderr,"er_report : NULL was passed for error_msg\n");
        
        }else{
            fprintf(flux," %s : %s at %s\n",repport_msg, str_flag(flag), error_msg);
    }
}

void er_error( const char * msg , err_code flag)
{ 
    //could be a macro 
    er_report(stdout, "error", msg, flag);
}

void er_warning(const char * msg , err_code flag)
{ 
    //could be a macro 
    er_report(stdout, "warning", msg, flag);
}

void skip_char(char chr , char** buff) {
    while (**buff == chr && **buff != '\0') {
        (*buff)++;
    }
} // static helper ; doesn't check for shit.

err_code fill_arr(char* str_source, uint32_t * arr_dest, uint32_t arr_size) {

    def_err_handler(! (str_source && arr_dest ), "fill_arr", ERR_NULL);

    uint32_t cpt = 0;

    char *start;
    char *end;
    start = end = str_source;

    do {
        start = end;

        uint32_t parsed_num = strtol(start, &end, DEC_BASE); // parses a number
        if (start != end) {
            arr_dest[cpt] = parsed_num;
        }

        skip_char(' ', &start); // goes to the next non space (next number)

        cpt++;
    } while (cpt < arr_size && start != end);

    const int ERR_BUFF_SIZE = 256;
    char * errbuf = calloc(ERR_BUFF_SIZE, sizeof(char));
    snprintf(errbuf, ERR_BUFF_SIZE-1, "fill_arr as=%u cpt=%u", arr_size, cpt);
    def_war_handler( (cpt != arr_size),errbuf, ERR_VAL);
    free(errbuf);

    return ERR_OK;
} // tested; seems ok

FILE* fopen_check(const char* filename, const char* mode){
    FILE* file = fopen(filename, mode);
    if (!file) {
        fprintf(stderr, "Cannot open file %s\n", filename);
    }
    return file;
}