#include "sparse_graph.h"

err_code init_graph(SPARSE_GRAPH* spg, uint32_t sea, uint32_t sra) {

	if (!spg) {
		return ERR_NULL;
	}

	spg->elems = calloc(sea, sizeof(uint32_t));
	if (!spg->elems) {
		return ERR_ALLOC;
	}

	spg->col_index = calloc(sea, sizeof(uint32_t));
	if (!spg->col_index) {
		return ERR_ALLOC;
	}

	spg->row_index = calloc(sra, sizeof(uint32_t));
	if (!spg->row_index) {
		return ERR_ALLOC;
	}
	spg->size_elems_arr = sea ; 
	spg->size_row_arr = sra ; 

	return ERR_OK;
} // tested ; seems ok

void free_graph(SPARSE_GRAPH* spg) {
	if (spg) {
		if (spg->elems) {
			free(spg->elems);
		}
		if (spg->col_index) {
			free(spg->col_index);
		}
		if (spg->row_index) {
			free(spg->row_index);
		}
		spg->elems = spg->col_index = spg->row_index = NULL;
		spg->size_row_arr = spg->size_elems_arr = 0;
	}
} // tested ; ok 


err_code get_link( int64_t * ret, SPARSE_GRAPH * spg , LINK lnk){
    if(!ret || !spg){ 
		return ERR_NULL;
	}
    if(spg->size_row_arr - 1 < lnk.row ) {
		*ret = -1; 
		return ERR_OK;
	}
    if(spg->size_elems_arr - 1 < lnk.col )  {
		*ret = -1; 
		return ERR_OK;
	}

    int64_t row_start = spg->row_index[lnk.row];
    int64_t row_end ;
    if(spg->size_row_arr - 1 == lnk.row ){
        row_end = spg->size_elems_arr ;
    }else{
        row_end = spg->row_index[lnk.row + 1 ];
    }
    
    for(uint32_t i = row_start ; i < row_end ; i++){
        if(spg->col_index[i] == lnk.col ){
            *ret = spg->elems[i] ;
            return ERR_OK ;
        }
    }
    *ret = -1 ;
    return ERR_OK ;
}//tested ; seems ok


err_code link_exists(bool* ret, SPARSE_GRAPH* spg, LINK lnk) {

	int64_t tmp;
	err_code failure = get_link(&tmp, spg, lnk);
	if (failure) {
		return failure;
	}

	*ret = (tmp != -1);
	return ERR_OK;
} // tested ok if get_link is ok

err_code set_link(SPARSE_GRAPH* spg, LINK lnk, uint32_t weight) {
	if(!spg){ 
		return ERR_NULL;
	}
    

    int64_t row_start = spg->row_index[lnk.row];
    int64_t row_end ;
    if(spg->size_row_arr - 1 == lnk.row ){
        row_end = spg->size_elems_arr ;
    }else{
        row_end = spg->row_index[lnk.row + 1 ];
    }
    
    for(uint32_t i = row_start ; i < row_end ; i++){
        if(spg->col_index[i] == lnk.col ){
            spg->elems[i] = weight ;
            return ERR_OK ;
        }
    }
    return ERR_OK ;
} // not tested

extern err_code write_graph(FILE* flux_dest, SPARSE_GRAPH* graph_source) {
	if (!(flux_dest && graph_source)) {
		return ERR_NULL;
	}

	fprintf(flux_dest, "%u %u\n", graph_source->size_elems_arr,
			graph_source->size_row_arr);

	for (uint32_t i = 0; i < graph_source->size_elems_arr; i++) {
		fprintf(flux_dest, "%u ", graph_source->elems[i]);
	}
	fprintf(flux_dest, "\n");
	for (uint32_t i = 0; i < graph_source->size_elems_arr; i++) {
		fprintf(flux_dest, "%u ", graph_source->col_index[i]);
	}
	fprintf(flux_dest, "\n");
	for (uint32_t i = 0; i < graph_source->size_row_arr; i++) {
		fprintf(flux_dest, "%u ", graph_source->row_index[i]);
	}
	fprintf(flux_dest, "\n");
	return ERR_OK;
} // not tested

static void skip_char(char chr , char** buff) {
	while (**buff == chr && **buff != '\0') {
		(*buff)++;
	}
} // static helper ; doesn't check for shit.

#define DEC_BASE 10
#define BUFF_SIZE 256
static void fill_arr(char* str_source, uint32_t* arr_dest, uint32_t arr_size) {

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
} // not tested; static helper ; doesn't check for shit
// prolly wrong

err_code read_graph(FILE* flux_source, SPARSE_GRAPH* graph_dest) {
	if (!(flux_source && graph_dest)) {
		return ERR_NULL;
	}

	char buff[BUFF_SIZE];
	fgets(buff, BUFF_SIZE, flux_source);
	skip_char(' ', (char**)&buff);

	char* end = (char*)&buff;
	char* start = (char*)&buff;

	uint32_t size_elems_arr = 0;
	size_elems_arr = strtol(start, &end, DEC_BASE);
	if (start == end) {
		return ERR_VAL;
	}
	start = end;

	uint32_t size_row_arr = 0;
	size_row_arr = strtol(start, &end, DEC_BASE);
	if (start == end) {
		return ERR_VAL;
	}

	err_code failure = init_graph(graph_dest, size_elems_arr, size_row_arr);
	if (failure) {
		return failure;
	}

	if (!fgets(buff, BUFF_SIZE, flux_source)) {
		return ERR_FORMAT;
	}
	fill_arr(buff, graph_dest->elems, size_elems_arr);

	if (!fgets(buff, BUFF_SIZE, flux_source)) {
		return ERR_FORMAT;
	}
	fill_arr(buff, graph_dest->col_index, size_elems_arr);

	if (!fgets(buff, BUFF_SIZE, flux_source)) {
		return ERR_FORMAT;
	}
	fill_arr(buff, graph_dest->row_index, size_row_arr);

	return ERR_OK;
} // not tested; 100% wrong
