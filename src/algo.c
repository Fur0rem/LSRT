#include "algo.h"
#include "utils.h"
#include <string.h>

err_code temporal_floy_warshall(LINK_STREAM * lst , DISTANCE_MATRIX_ARRAY * dma){
    def_err_handler(!(lst && dma), "temporal_floyd_warshall", ERR_NULL);

    err_code failure ;

    for(uint32_t i = 0 ; i < dma->nb_matrixes ; i ++){ //initializes matrixes to + infinity
        memset( dma->matrixes[i].values , UINT8_MAX,  dma->matrixes[i].rows * dma->matrixes[i].rows * sizeof(uint8_t));
    }

    for(uint32_t row = 0 ; row < lst->sparse_graph.size_row_arr ; row ++){
    //initializes links to weight(u,v)
        for(uint32_t col = lst->sparse_graph.row_index[row] ; col <  lst->sparse_graph.row_index[row+1] ; col++){
            
            int64_t ret ; 
            LINK lnk = {col,row};
    
            get_link(&ret, &(lst->sparse_graph), lnk );
            for(uint32_t time = 0 ; time < dma->nb_matrixes ; time++){
                bool deleted ; 
                is_deleted(&deleted, &(lst->deleted_links), row+col, time);  
                //is deleted call might be wrong
                if(!deleted){
                    set_elem_dma(dma, ret, time, lnk.row, lnk.col);
                }
            }

        }
    }

    //intializes edge values (ie : mat[v][v]) to zero
    for(uint32_t mat = 0 ; mat < dma->nb_matrixes ; mat++){
        for(uint32_t row = 0 ; row < dma->matrixes[mat].rows ; row ++){
            set_elem_dma(dma,0, mat, row, row);
        }
    }

    
    //the big floyd warshall O(n^3) loop goes here
    /*
    before coding this loop I wanna make sure how I 
    iterate in "time" bc I don't want to mess this up 
    I think I should iterate from time = max to time = 0 
    */


    return ERR_OK;
}//not done
//doesnt check function calls atm ; 

