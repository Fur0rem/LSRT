#include "algo_parrallel.h"
#include "common.h"
#include "link_stream.h"
#include "matrix.h"
#include "utils.h"
#include "algo.h"
#include "thpool.h"

#include <stdint.h>
#include <stdlib.h>
#include <sys/types.h>


/*
THIS VERY MUCH IS A WIP FILE 
ITS VERY VERY VERY MESSY 
AND IT DOESNT WORK **AT ALL**
DO NOT TRY TO USE ANY OF THE FUNCTIONS
PLEASE DON'T.
UNDER ANY CIRCUMSTANCES 
I BEG YOU 
DO NOT LOOK AT IT EITHER 

- K.avi :) 
*/

// OK so ACTUALLY I'll have to implement 
// a tiled FW in order to juice the 
//multi-threading as much as possible
//Man I thought the thread pool part 
//was the annoying one smh 


//local structure -> start reference 
//to a dma index to write to 
//length -> number of values to write 
typedef struct S_MATSECTION{
    DISTANCE_MATRIX * dm ;
    uint32_t subsection_start_index ;
    uint32_t subsection_length ;
    
}s_matsection;
//each subsection length and start index should be 
//computed only once ; the only thing that 
//will change is the dma_ref

typedef struct S_MATSECTION_TAB{
    uint32_t size ; 
    
    DISTANCE_MATRIX* ref_dm ;
    uint32_t* arr_subsection_start_index ; 
    uint32_t* arr_subsection_length ;
    
}s_matsection_tab ; 


static err_code init_matsection_tab(s_matsection_tab * tab,
    DISTANCE_MATRIX_ARRAY * dma, uint32_t time){

    def_err_handler(!tab, "init_matsection_tab", ERR_NULL);
    def_err_handler(!dma, "init_matsection_tab", ERR_VAL);

    uint32_t size = dma->matrixes[0].cols * dma->matrixes[0].rows / global_nb_threads ;

    tab->ref_dm = &dma->matrixes[time] ;

    tab->size = size ;

    return ERR_OK; 
}//only initializes the matsection tab ; each matsection has to be initialized 
//by hand and then updated to the DMA_TAB at each time and so on 
//bah gawd I have an idea
//Ok my idea is shit; fuck it we ballin
//NOT DONE 

void * thwork_matrix(void * arg){
    s_matsection * section = (s_matsection *) arg ; 
    if(!section){
        return NULL ; 
    }
    //do the work 
    //floyd_warshall(section->dma, section->start_index, section->length);
    return NULL ; 
}//subroutine executed by the worker threads
//NOT DONE 

static err_code split_matrix(DISTANCE_MATRIX_ARRAY * dma,  s_matsection * sections){
    return ERR_OK ;
}//no? 

static err_code launch_batch_ite(DISTANCE_MATRIX * dist_mat , S_THPOOL * thpool){

    return ERR_OK ; 
}//not done 

err_code temporal_floyd_warshall_multi_threaded(LINK_STREAM * lst , DISTANCE_MATRIX_ARRAY * dma){

    S_THPOOL tpool ; 
    err_code failure = thpool_init(&tpool); 
    def_err_handler(failure , "tmp_dw_mt", failure); 

    thpool_start(&tpool);

    for(uint32_t i = dma->nb_matrixes ; i > 0 ; i--){

        launch_batch_ite(&dma->matrixes[i],&tpool);
        thpool_wait_for_all(&tpool);
    }

    thpool_destroy(&tpool); 
    return ERR_OK; 
}//not done 

