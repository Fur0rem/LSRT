#ifndef LK_STREAM_H 
#define LK_STREAM_H 

#include "sparse_graph.h"
#include "deleted_links.h"

typedef struct s_link_stream{ 

    SPARSE_GRAPH sparse_graph ; 
    DELETED_LINKS_TAB deleted_links ; 
    
}LINK_STREAM; 
#define declare_link_stream(lks) LINK_STREAM lks = { {0, NULL, NULL, 0, NULL}, {0, NULL}};
/*
    simple wrapper around sparse graph and 
    deleted links to make it more handy :)

    when calling functions on the graph or deleted links tab 
    of a link stream ; directly call functions defined in their API. 

    When passing a linkstream as arg / parsing it , use these functions instead
*/

extern err_code parse_link_stream(LINK_STREAM * lks, FILE * graph_source, FILE * deleted_links_source );
/*
    parses a link stream from the source of it's graph and deleted links 
*/
extern void free_link_stream(LINK_STREAM * lks); 
/*
    frees the graph and sparse deleted links struct of 
    an lks
*/

#endif 
