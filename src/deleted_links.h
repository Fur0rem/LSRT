#ifndef DELETED_LINKS_H 
#define DELETED_LINKS_H 

#include "common.h"


typedef struct uarr32{
    uint32_t size; 
    uint32_t * elems; 
}UARR_32;

typedef struct delinks{

    uint32_t size ; 
    UARR_32 * elems ; //links are the indexes; elems[i] stores the times when the link i is deleted
}DELETED_LINKS_TAB ;

extern err_code init_dlt(DELETED_LINKS_TAB * dlt, uint32_t size );
extern void free_dlt(DELETED_LINKS_TAB * dlt, uint32_t size);

extern err_code write_dlt(FILE * flux, DELETED_LINKS_TAB * dlt); 
extern err_code read_dlt(FILE * flux, DELETED_LINKS_TAB * dlt); 

extern err_code is_deleted(bool * ret, DELETED_LINKS_TAB * dlt, uint32_t link, uint32_t time);


#endif
