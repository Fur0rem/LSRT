#include <stdint.h>
#include <stdio.h>
#include "src/algo.h"
#include "src/common.h"
#include "src/utils.h"
#include <bits/getopt_core.h>//should not be necessary

int main(int argc, char ** argv) {

	uint8_t opt_mask = 0 ;
	FILE * flux = stdout ;
	uint8_t opt_shift = 0 ; 

	int chr;
    while ((chr = getopt(argc, argv, "hf:d")) != -1) {
        
        switch (chr) {
        case 'h': //help
            opt_mask |= 1; 
			opt_shift++;
            break;
        case 'f' : //output file
			opt_mask |= 1<<1; 
			opt_shift+=2;
			flux = fopen(optarg,"a");
			break;
		case 'd' : //output file
			opt_mask |= 1<<2; 
			opt_shift++;
			break;

        case '?':    
            fprintf(stderr, "Unknown option character %c.", optopt);          
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
	FILE * fgraph = fopen(argv[1+opt_shift], "r");
	FILE * fattack = fopen(argv[2+opt_shift] , "r");

	err_code failure = parse_link_stream(&lks, fgraph, fattack );
	def_err_handler(failure, "main parse_link", failure);

	fclose(fgraph); 
	fclose(fattack);
	//initializes the matrixes 
	declare_dma(dma); 	
	failure = init_dma(&dma, lks.deleted_links.nb_it, lks.sparse_graph.size_row_arr, lks.sparse_graph.size_row_arr); 
	def_err_handler(failure, "main init_dma", failure);
	//starts the algorithm
	failure = temporal_floyd_warshall(&lks, &dma);
	def_err_handler(failure, "main temporal_floyd_warshall", failure);
	//sums every little square in the matrix and writes results
	double ret ; 
	uint64_t reachables; 
	failure = sum_dma(&ret, &reachables,  &dma);
	def_err_handler(failure, "main sum_dma", failure);
	fprintf(flux, "%lf %lu\n", ret, reachables);
	//cleans stuff up
	if(flux != stdout ){
		fclose(flux);
	}
	if(opt_mask &  (1<<2) ){
		FILE * f = fopen("dma_dump","a");
		fprint_dma(f, &dma);
		fclose(f);
	}

	free_dma(&dma);
	free_link_stream(&lks);
	return 0;
}
