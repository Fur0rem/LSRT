#ifndef LK_STREAM_H 
#define LK_STREAM_H 

#include "sparse_graph.h"
#include "deleted_links.h"

typedef struct s_link_stream{ 

    SPARSE_GRAPH sparse_graph ; 
    DELETED_LINKS_TAB deleted_links ; 
}LINK_STREAM; 

extern err_code parse_link_stream(LINK_STREAM * lks, FILE * graph_source, FILE * deleted_links_source );
extern void free_link_stream(LINK_STREAM * lks); 

#endif 
