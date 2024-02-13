#include <stdio.h>
#include "src/algo.h"
#include "src/common.h"
#include "src/utils.h"
#include <bits/getopt_core.h>//should not be necessary

int main(int argc, char ** argv) {

	uint8_t opt_mask = 0 ;
	FILE * flux = stdout ;

	char chr;
    while ((chr = getopt(argc, argv, "hf:")) != -1) {
        
        switch (chr) {
        case 'h': //help
            opt_mask |= 1; 
            break;
        case 'f' : //output file
			opt_mask |= 1<<1; 
			flux = fopen(optarg,"a");

        case '?':    
            fprintf(stderr, "Unknown option character `\\x%x'.", optopt);          
            break;
        default:
            abort();
        }
    }//parses options

	if(opt_mask & (uint8_t) 1 || argc < 3){
		fprintf(flux,"usage : lsrt [graph_file] [attack_file]\n");
		return ERR_OK;
	}//checks for good usage 

	//parses link stream from files
	declare_link_stream(lks);
	FILE * fgraph = fopen(argv[1], "r");
	FILE * fattack = fopen(argv[2] , "r");

	err_code failure = parse_link_stream(&lks, fgraph, fattack );
	def_err_handler(failure, "main parse_link", failure);

	fclose(fgraph); 
	fclose(fattack);
	//initializes the matrixes 
	declare_dma(dma); 	
	failure = init_dma(&dma, lks.deleted_links.size, lks.sparse_graph.size_row_arr, lks.sparse_graph.size_row_arr); 
	def_err_handler(failure, "main init_dma", failure);
	//starts the algorithm
	failure = temporal_floyd_warshall(&lks, &dma);
	def_err_handler(failure, "main temporal_floyd_warshall", failure);
	//sums every little square in the matrix and writes results
	uint64_t ret ; 
	failure = sum_dma(&ret,  &dma);
	def_err_handler(failure, "main sum_dma", failure);
	fprintf(flux, "%lu\n", ret);
	//cleans stuff up
	if(flux != stdout ){
		fclose(flux);
	}
	free_dma(&dma);
	free_link_stream(&lks);
	return 0;
}
