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

#define declare_dlt(dlt) DELETED_LINKS_TAB (dlt) = {0,NULL};
/*
    macro function to cleanly declare a DLT ; pass an unused name in the 
    namespace
*/
extern err_code init_dlt(DELETED_LINKS_TAB * dlt, uint32_t size );
/*
    dlt -> not null 

    initializes the dlt elems to size entry and sets dlt->size to size. 
    doesn't initializes the entries of dlt->elems; 
    doesn't check for memleak 
*/
extern void free_dlt(DELETED_LINKS_TAB * dlt);
/*
    frees dlt and it's entries if they're not null and sets it to 
    null.
*/

extern err_code write_dlt(FILE * flux, DELETED_LINKS_TAB * dlt);
/*
    flux & dlt -> not null 

    writes a dlt in flux
*/ 
extern err_code read_dlt(FILE * flux, DELETED_LINKS_TAB * dlt); 
/*
    flux & dlt -> not null 
    INITIALIZES a dlt and parses it from flux .
*/

extern err_code is_deleted(bool * ret, DELETED_LINKS_TAB * dlt, uint32_t link, uint32_t time);
/*
    ret & dlt -> not null & initialized
    returns true if a link is deleted at time t in dlt ; false otherwise .
*/

#endif
