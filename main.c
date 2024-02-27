#include <stdint.h>
#include <stdio.h>
#include "src/algo.h"
#include "src/common.h"
#include "src/utils.h"
#include <bits/getopt_core.h>
#include <stdlib.h>
#include <pthread.h>
#include <string.h>


const int FLAG_HELP = 1 << 0;
const int FLAG_FLUX = 1 << 1;
const int FLAG_DEBUG = 1 << 2;

double process_attack(char* attack_file, const char* graph_content, uint32_t budget, int opt_mask) {
    printf("budget: %u\n", budget);

    // first attack
    char* attack_filename = calloc(256, sizeof(char));
    snprintf(attack_filename, 255, "%s_%u", attack_file, budget);
    printf("attack_filename: %s\n", attack_filename);

    // Parse linked stream from files
    declare_link_stream(lks);
    FILE* fattack = fopen_check(attack_filename, "r");

	// TODO : utiliser un const char* plutot que cette atrocité c'est quoi cette merde
	FILE* fgraph = fmemopen((void*)graph_content, strlen(graph_content), "r");

    err_code failure = parse_link_stream(&lks, fgraph, fattack );
    def_err_handler(failure, "main parse_link", failure);

    fclose(fgraph); 
    fclose(fattack);

    // Initialize Distance Matrix Array
    declare_dma(dma);     
    failure = init_dma(&dma, lks.deleted_links.nb_it, lks.sparse_graph.size_row_arr, lks.sparse_graph.size_row_arr); 
    def_err_handler(failure, "main init_dma", failure);

    // Main algorithm
    failure = temporal_floyd_warshall(&lks, &dma);
    def_err_handler(failure, "main temporal_floyd_warshall", failure);

    // Calculate sum (the metric we want to compute)
    double ret; 
    uint64_t reachables; 
    failure = sum_dma(&ret, &reachables, &dma);

    printf("ret: %lf\n", ret);
    def_err_handler(failure, "main sum_dma", failure);

    if (opt_mask & FLAG_DEBUG) {
        FILE* fdma = fopen_check("dma_dump","a");
        fprint_dma(fdma, &dma);
        fclose(fdma);
    }

    free_dma(&dma);
    free_link_stream(&lks);
    free(attack_filename);

	return ret;
}

struct process_attack_args {
	double* all_answers;
	char* attack_file;
	char* graph_content;
	uint32_t budget;
	int opt_mask;
	int index;
};

void* process_attack_thread(void* args) {
	struct process_attack_args* paa = (struct process_attack_args*) args;
	paa->all_answers[paa->index] = process_attack(paa->attack_file, paa->graph_content, paa->budget, paa->opt_mask);
	return NULL;
}

int main(int argc, char* argv[]) {

	uint8_t opt_mask = 0;
	FILE * flux = stdout;
	uint8_t opt_shift = 0; 
	int chr;

	char* output_file;

	// Parse options
    while ((chr = getopt(argc, argv, "hf:d")) != -1) {
        switch (chr) {

			case 'h':
				opt_mask |= FLAG_HELP;
				opt_shift++;
				break;
			case 'f' :
				opt_mask |= FLAG_FLUX; 
				opt_shift += 2;
				output_file = optarg;
				flux = fopen_check(optarg,"a");
				break;
			case 'd' :
				opt_mask |= FLAG_DEBUG;
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
		fprintf(flux,"Usage : lsrt <graph_file> <attack_file> <output_file> <budgets>\n");
		return ERR_OK;
	}

	char* graph_file = argv[1+opt_shift];
	char* attack_file = argv[2+opt_shift];
	char* budgets = argv[3+opt_shift];
	printf("graph_file: %s\n", graph_file);
	printf("attack_file: %s\n", attack_file);
	printf("output_file: %s\n", output_file);
	printf("budgets: %s\n", budgets);

	// Parse the budgets, the first number is the number of budgets
	uint32_t nb_budgets = strtol(budgets, &budgets, DEC_BASE);
	uint32_t* budgets_arr = calloc(nb_budgets, sizeof(uint32_t));
	fill_arr(budgets, budgets_arr, nb_budgets);

	// Allocate memory for the answers
	double* all_answers = calloc(nb_budgets, sizeof(double));
	pthread_t* threads = calloc(nb_budgets, sizeof(pthread_t));

	// Open the graph file
	FILE* fgraph = fopen_check(graph_file, "r");
	char* graph_content = calloc(1024*1024, sizeof(char));
	size_t size = fread(graph_content, sizeof(char), 1024*1024, fgraph);
	graph_content[size] = '\0';

	// Process the attacks
    for (int i = 0; i < nb_budgets; i++) {
        printf("budgets_arr[%d]: %u\n", i, budgets_arr[i]);
        struct process_attack_args paa;
        paa.all_answers = all_answers;
        paa.attack_file = attack_file;
        paa.graph_content = graph_content;
        paa.budget = budgets_arr[i];
        paa.opt_mask = opt_mask;
        paa.index = i;
        pthread_create(&threads[i], NULL, process_attack_thread, &paa);
    }

	// Wait for all threads to finish
	for (int i = 0; i < nb_budgets; i++) {
		pthread_join(threads[i], NULL);
	}

	fclose(fgraph);

	// Write the answers to the output file
	FILE* foutput = fopen_check(output_file, "w");
	for (int i = 0; i < nb_budgets; i++) {
		fprintf(foutput, "%u %lf\n", budgets_arr[i], all_answers[i]);
	}

	// Clean up
		if (flux != stdout) {
			fclose(flux);
		}
return 0;
}
