#include "link_stream.h"
#include "utils.h"

extern err_code parse_link_stream(LINK_STREAM * lks, FILE * graph_source, FILE * deleted_links_source ){
    def_err_handler(!(lks && graph_source && deleted_links_source), "parse_link_stream", ERR_NULL);

    err_code failure = read_graph(graph_source, &(lks->sparse_graph));
    def_err_handler(failure ,"parse_link_stream", failure);

    failure = read_dlt(deleted_links_source, &(lks->deleted_links));
    def_err_handler(failure ,"parse_link_stream", failure);

    return ERR_OK ;
}//not tested; ok 

extern void free_link_stream(LINK_STREAM * lks){
    free_graph(&(lks->sparse_graph));
    free_dlt(&(lks->deleted_links));
}//not tested ; ok
