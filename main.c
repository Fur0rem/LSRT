#include <stdint.h>
#include <stdio.h>
#include "src/algo.h"
#include "src/common.h"
#include "src/utils.h"
#include <bits/getopt_core.h>
#include <time.h>

#define FLAG_HELP 1 << 0
#define FLAG_FLUX  1 << 1
#define FLAG_DEBUG  1 << 2
#define FLAG_BENCHMARK 1 << 3

int main(int argc, char* argv[]) {

	uint8_t opt_mask = 0;
	FILE * flux = stdout;
	uint8_t opt_shift = 0; 
	int chr;

	// Parse options
    while ((chr = getopt(argc, argv, "hf:db")) != -1) {
        switch (chr) {

			case 'h':
				opt_mask |= FLAG_HELP;
				opt_shift++;
				break;
			case 'f' :
				opt_mask |= FLAG_FLUX; 
				opt_shift += 2;
				flux = fopen_check(optarg,"a");
				break;
			case 'd' :
				opt_mask |= FLAG_DEBUG;
				opt_shift++;
				break;
			case 'b' : 
				opt_mask |= FLAG_BENCHMARK;
				opt_shift++;
				break;

			case '?':    
				fprintf(stderr, "Unknown option character %c.", optopt);          
				break;
			default:
				abort();
        }
    }

	// Check for good usage
	if (opt_mask & FLAG_HELP || optind != 3) {
		fprintf(flux,"Usage : lsrt <graph_file> <attack_file>\n");
		return ERR_OK;
	}
	
	// Parse linked stream from files
	declare_link_stream(lks);
	FILE* fgraph = fopen_check(argv[1+opt_shift], "r");
	FILE* fattack = fopen_check(argv[2+opt_shift] , "r");


	time_t total;
	time_t start; 
	time_t end;
	if (opt_mask & FLAG_BENCHMARK) {	
		time(&start);
		total = start;
	}

	err_code failure = parse_link_stream(&lks, fgraph, fattack );
	def_err_handler(failure, "main parse_link", failure);

	if(opt_mask & FLAG_BENCHMARK){	
		time(&end);
		fprintf(flux,"parse_link_stream tot time : %lu ", start - end);
	}

	fclose(fgraph); 
	fclose(fattack);


	// Initialize Distance Matrix Array
	declare_dma(dma); 	


	if (opt_mask & FLAG_BENCHMARK) {
		time(&start);
	}
	failure = init_dma(&dma, lks.deleted_links.nb_it, lks.sparse_graph.size_row_arr, lks.sparse_graph.size_row_arr); 
	def_err_handler(failure, "main init_dma", failure);
	
	if(opt_mask & FLAG_BENCHMARK){	
		time(&end);
		fprintf(flux,"init_dma tot time : %lu ", start - end);
	}

	// Main algorithm
	if (opt_mask & FLAG_BENCHMARK) {
		time(&start);
	}

	failure = temporal_floyd_warshall(&lks, &dma);
	def_err_handler(failure, "main temporal_floyd_warshall", failure);

	if(opt_mask & FLAG_BENCHMARK){	
		time(&end);
		fprintf(flux,"temporal_floyd_warshall tot time : %lu ", start - end);
	}
	

	// Calculate sum (the metric we want to compute)
	double ret; 
	uint64_t reachables; 

	if (opt_mask & FLAG_BENCHMARK) {
		time(&start);
	}

	failure = sum_dma(&ret, &reachables, &dma);
	def_err_handler(failure, "main sum_dma", failure);

	if(opt_mask & FLAG_BENCHMARK){	
		time(&end);
		fprintf(flux,"sum_dma tot time : %lu ", start - end);
	}
	
	fprintf(flux, "%lf %lu\n", ret, reachables);

	// Clean up
	if (flux != stdout) {
		fclose(flux);
	}

	if (opt_mask & FLAG_DEBUG) {
		FILE* fdma = fopen_check("dma_dump","a");
		fprint_dma(fdma, &dma);
		fclose(fdma);
	}

	free_dma(&dma);
	free_link_stream(&lks);

	if(opt_mask & FLAG_BENCHMARK){	
		time(&end);
		fprintf(flux,"total time : %lu ", total - end);
	}

	return 0;
}
