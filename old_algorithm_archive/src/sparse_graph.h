#ifndef SPARSE_GRAPH_H
#define SPARSE_GRAPH_H

#include "common.h"

typedef struct s_gsparse {

	uint32_t size_elems_arr;
	uint32_t* elems;
	uint32_t* col_index;

	uint32_t size_row_arr;
	uint32_t* row_index;

} SPARSE_GRAPH;
// sparse csgraph structure used in the code
#define declare_sparse_graph(graph) SPARSE_GRAPH(graph) = {0, NULL, NULL, 0, NULL};
/*
	macro to initialize the fields of a graph when declaring it
*/

typedef struct s_link {
	uint32_t row;
	uint32_t col;
} LINK;
// little link structure for convenience
#define declare_link(link) LINK(link) = {0, 0};

extern err_code write_graph(FILE* flux_dest, SPARSE_GRAPH* graph_source);
/*
	graph_source, flux_dest -> not null
	writes graph_source to flux_dest
*/

extern err_code read_graph(FILE* flux_source, SPARSE_GRAPH* graph_dest);
/*
	graph_source, flux_dest -> not null
	read graph_dest from flux_dest
*/

extern err_code link_exists(bool* ret, SPARSE_GRAPH* spg, LINK lnk);
/*
	ret,spg -> not null
	lnk.i and lnk.j must be less than nb_elems

	sets ret to true if a link (i,j) exists in spg, sets it to false otherwise.
*/

extern err_code get_link(int64_t* ret, SPARSE_GRAPH* spg, LINK lnk);
/*
	ret,spg -> not null
	lnk.i and lnk.j must be less than nb_elems

	sets ret to the value of the link i,j if it exists in spg ,
	sets it to -1 otherwise .
*/

extern void free_graph(SPARSE_GRAPH* spg);
/*
	spg -> *
	frees spg if it's not null
*/

#ifdef debug
extern err_code init_graph(SPARSE_GRAPH* spg, uint32_t sea, uint32_t sra);
/*
	spg -> not null

	initializes the graph spg, the array elems and col_index are initialized to sea
	the array row_index is initialized to sra

	throws errors on alloc failure
*/

extern err_code set_link(SPARSE_GRAPH* spg, LINK lnk, uint32_t weight);
/*
	spg -> not null
	lnk.i and lnk.j must be less than nb_elems

	sets the link lnk to the weight passed as argument if it exists,
	overwrites the previous weight if it's already set
*/
#endif

#endif
