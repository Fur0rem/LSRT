#include "algo.h"
#include "sparse_graph.h"
#include "utils.h"
#include <stdint.h>
#include <string.h>


static err_code prepare_tfw(LINK_STREAM * lst, DISTANCE_MATRIX_ARRAY * dma){
    def_err_handler(!(lst && dma), "temporal_floyd_warshall", ERR_NULL);
    

    //write_graph(stdout, &lst->sparse_graph);
    for(uint32_t i = 0 ; i < dma->nb_matrixes ; i ++){ //initializes matrixes to + infinity
        memset( dma->matrixes[i].values , UINT8_MAX,  dma->matrixes[i].rows * dma->matrixes[i].rows * sizeof(uint8_t));
    }
    err_code failure ;

    //initializes links to weight(u,v)
    for(uint32_t row = 0 ; row < lst->sparse_graph.size_row_arr ; row ++){
        //end_it allows to not overflow the array
        uint32_t end_it ; // (row + 1 == lst->sparse_graph.size_row_arr ) ?
                          //lst->sparse_graph.size_elems_arr : lst->sparse_graph.row_index[row+1];

        if(row + 1 == lst->sparse_graph.size_row_arr){
            end_it = lst->sparse_graph.size_elems_arr;
        }else{
            end_it = lst->sparse_graph.row_index[row+1];
        }

        //printf("row=%u end_it=%u, lst.spg.row[row]=%u\n",row, end_it, lst->sparse_graph.row_index[row]);
        for(uint32_t col_index = lst->sparse_graph.row_index[row] ; col_index <  end_it ; col_index ++){
            //fetch column number from the index
            uint32_t col = lst->sparse_graph.col_index[col_index];
            LINK lnk; 
            lnk.col = col; 
            lnk.row = row ;
            

            int64_t ret ; 
            //printf("link={%u,%u}\n", col, row);
            failure = get_link(&ret, &(lst->sparse_graph), lnk );//fetches the weight on the link
            def_err_handler(failure, "prepare_ftw", failure);
            
            //sets dma->mat[t].values[i][j] to ret if it's not deleted
            for(uint32_t time = 0 ; time < dma->nb_matrixes ; time++){
                bool deleted = true ; 
                //is_deleted call might be wrong
                failure = is_deleted(&deleted, &(lst->deleted_links), col_index, time);  
                def_err_handler(failure, "prepare_ftw", failure);
                //set elem dma might be wrong
                if(!deleted && (ret + time < dma->nb_matrixes)){

                    //printf("setelemdma ret=%u time=%u lnk={%u,%u}\n", (uint32_t)ret, time , lnk.row, lnk.col);
                    failure = set_elem_dma(dma, (uint32_t) ret, time, lnk.row, lnk.col);
                    def_err_handler(failure, "prepare_ftw", failure);
                }else{
                    //printf("deleted=%u , ret=%u, lnk={%u,%u}\n", deleted, ret,lnk.row, lnk.col );
                }
            }
        }
    }
    //intializes edge values (ie : mat[v][v]) to zero
    for(uint32_t mat = 0 ; mat < dma->nb_matrixes ; mat++){
        for(uint32_t row = 0 ; row < dma->matrixes[mat].rows ; row ++){
            failure = set_elem_dma(dma,1, mat, row, row);//one to self bc you still advance in time
            def_err_handler(failure, "prepare_ftw", failure);
        }
    }
    return ERR_OK ;
}//this SEEMS ok. It might not be though.

err_code temporal_floyd_warshall(LINK_STREAM * lst , DISTANCE_MATRIX_ARRAY * dma){
    def_err_handler(!(lst && dma), "temporal_floyd_warshall", ERR_NULL);

    err_code failure = prepare_tfw(lst, dma); 
    def_err_handler(failure, "temporal_floyd_warshall", failure);    

    //the big temporal floyd warshall loop
    for(int64_t time = dma->nb_matrixes - 1 ; time > -1 ; time--){
        for(uint32_t k = 0 ; k < dma->matrixes->rows; k++){
            for(uint32_t  i = 0 ; i < dma->matrixes->rows ; i++){
                for(uint32_t  j = 0 ; j < dma->matrixes->rows ; j++){
                    
                    //check that the distance isn't > than the endtime
                    uint16_t dist_ij = UINT8_MAX ;
                    
                    failure = get_elem_dma(dma, (uint8_t *)&dist_ij, time, i,j );
                    def_err_handler(failure, "temporal_floyd_warshall 1", failure);   
                       
                    uint16_t dist_ik = UINT8_MAX ;

                    failure = get_elem_dma(dma, (uint8_t *)&dist_ik, time, i,k );
                    def_err_handler(failure, "temporal_floyd_warshall 2", failure);   
                    //recupere distance a time + dist(i,k)

                    uint16_t dist_kj = UINT8_MAX ;
                    if(time + dist_ik < dma->nb_matrixes){//checks that time + dist_ik is not after the last time
                        //printf("dist_kj checked\n");
                        failure = get_elem_dma(dma, (uint8_t * )&dist_kj, time + dist_ik , k,j );
                        def_err_handler(failure, "temporal_floyd_warshall 3", failure); 
                    }  
                    //printf("time=%u (i,j,k)=(%u,%u,%u)\ndist_ij=%u, distik=%u, dist_kj=%u\n",time,i,j,k,dist_ij, dist_ik , dist_kj);

                    if( (dist_ij > (dist_ik + dist_kj)) && (time + dist_ik + dist_kj) < dma->nb_matrixes  ){
                        //failure = set_elem_dma(dma, dist_ik + dist_kj, time, i, j);
                       
                        dma->matrixes[time].values[ i * dma->matrixes[time].cols + j] = dist_ik + dist_kj ;

                        //def_err_handler(failure, "temporal_floyd_warshall", failure);   
                        //printf("time=%u (i,j,k)=(%u,%u,%u)\ndist_ij=%u, distik=%u, dist_kj=%u\n",time,i,j,k,dist_ij, dist_ik , dist_kj);

                    }
                }   
            }
        }
        printf("iteration number : i=%ld\n",time  );
    }
    return ERR_OK;
}//tested ; somewhat ok ; needs more testing
//does it handle the waiting case ??? 
//i'm really unsure

err_code sum_dma(double * ret_sum, uint64_t * ret_reachables,  DISTANCE_MATRIX_ARRAY * dma){
    def_err_handler(!(ret_sum && dma && ret_reachables), "sum_dma", ERR_NULL);

    double sum = 0; 
    uint64_t reachables = 0 ; 
    for(uint32_t i = 0 ; i < dma->nb_matrixes ; i++){
        for(uint32_t j = 0 ; j < dma->matrixes[i].cols * dma->matrixes[i].rows ; j++){
            
            if( (dma->matrixes[i].values[j] != UINT8_MAX) && (dma->matrixes[i].values[j]) && i!=j ){
                sum += (double)((double)1/(double)(dma->matrixes[i].values[j])) ;
                reachables++;
            }
        }
    }
    sum -= dma->matrixes[0].rows ;
    sum /=  dma->nb_matrixes *(((dma->matrixes[0].rows) * (dma->matrixes[0].rows - 1) ));
    //sum normalized by time * (|N| * |N-1|) with G=(V,N)

    reachables -= dma->matrixes[0].rows ;
    //reachables /= (((dma->matrixes[0].rows) * (dma->matrixes[0].rows - 1) ));
   
    *ret_sum = sum ; 
    *ret_reachables = reachables ;
    return ERR_OK ;
}//not tested ; should be okay though 
