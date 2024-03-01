#include "algo_parrallel.h"
#include "common.h"
#include "link_stream.h"
#include "matrix.h"
#include "utils.h"
#include "algo.h"
#include <bits/pthreadtypes.h>
#include <stdint.h>


static err_code compute_min_distance( DISTANCE_MATRIX_ARRAY * dma,uint32_t time, uint32_t i , uint32_t j , uint32_t k){
    //check that the distance isn't > than the endtime
    uint16_t dist_ij = UINT8_MAX ;
                    
    err_code failure = get_elem_dma(dma, (uint8_t *)&dist_ij, time, i,j );
    def_err_handler(failure, "temporal_floyd_warshall 1", failure);   
                       
    uint16_t dist_ik = UINT8_MAX ;

    failure = get_elem_dma(dma, (uint8_t *)&dist_ik, time, i,k );
    def_err_handler(failure, "temporal_floyd_warshall 2", failure);   
    //recupere distance a time + dist(i,k)

    uint16_t dist_kj = UINT8_MAX ;
    if(time + dist_ik < dma->nb_matrixes){//checks that time + dist_ik is not after the last time
        failure = get_elem_dma(dma, (uint8_t * )&dist_kj, time + dist_ik , k,j );
        def_err_handler(failure, "temporal_floyd_warshall 3", failure); 
    }  

    if( (dist_ij > (dist_ik + dist_kj)) && (time + dist_ik + dist_kj) < dma->nb_matrixes  ){   
        dma->matrixes[time].values[ i * dma->matrixes[time].cols + j] = dist_ik + dist_kj ;
    }
    return ERR_OK; 
}


err_code mat_calc_mt( LINK_STREAM * lst, DISTANCE_MATRIX_ARRAY * dma, uint32_t time ){
    /*
        multithreaded version of matrix update; 
        each thread should compute it's fraction of the matrix. 

    */
    return ERR_OK;
}


typedef struct thread_tab{
    uint32_t size ; 
    pthread_t * elems; 
}THREADS_TAB; 

err_code init_thtab(THREADS_TAB * thtab){

}

err_code temporal_floyd_warshall_multi_threaded(LINK_STREAM * lst , DISTANCE_MATRIX_ARRAY * dma){


    def_err_handler(!(lst && dma), "temporal_floyd_warshall", ERR_NULL);

    err_code failure = prepare_tfw(lst, dma); 
    def_err_handler(failure, "temporal_floyd_warshall", failure);    

    //init threads here

    //the big temporal floyd warshall loop
    for(int64_t time = dma->nb_matrixes - 1 ; time > -1 ; time--){
        /*for(uint32_t k = 0 ; k < dma->matrixes->rows; k++){
            for(uint32_t  i = 0 ; i < dma->matrixes->rows ; i++){
                for(uint32_t  j = 0 ; j < dma->matrixes->rows ; j++){
                   compute_min_distance(dma,  time, i, j, k) ;
                }   
            }
        }*/
        printf("iteration number : i=%ld\n",time  );
    }
    return ERR_OK;
}
