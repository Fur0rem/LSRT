#include "job_deque.h"
#include "common.h"
#include "utils.h"
#include <string.h>

#define DEF_QUEUE_SIZE 64
#define QUEUE_REALLOC_COEFF 2

err_code task_queue_init(S_TASKQUEUE * queue){
    def_err_handler(!queue, "thqueue_create", ERR_NULL);

    queue->tasks = (S_TASK *)calloc(DEF_QUEUE_SIZE , sizeof(S_TASK));
    def_err_handler(!queue->tasks, "thqueue_create", ERR_ALLOC);

    queue->size = 0 ; 
    queue->max = DEF_QUEUE_SIZE;
    queue->start = 0;

    return ERR_OK ; 
}//tested works

static err_code task_queue_realloc(S_TASKQUEUE * queue){
    def_err_handler(!queue, "thqueue_realloc", ERR_NULL);

    queue->tasks = realloc(queue->tasks , queue->max * QUEUE_REALLOC_COEFF * sizeof(S_TASK));
    def_err_handler(!queue->tasks, "thqueue_realloc", ERR_ALLOC);


    if( queue->start + queue->size  > queue->max){
        uint32_t shift = queue->start + queue->size - queue->max;
        uint32_t new_start = queue->max * QUEUE_REALLOC_COEFF - shift;
       // printf("shift %d, start %d  newstart %d, size %d max %d, newmax %d\n", shift, queue->start, new_start, queue->size, queue->max,queue->max * QUEUE_REALLOC_COEFF);
        memmove( &queue->tasks[new_start], &queue->tasks[queue->start], shift * sizeof(S_TASK));
        queue->start += new_start;
    }
   
    queue->max *= QUEUE_REALLOC_COEFF;

    return ERR_OK;
}//tested ; ok :)

err_code add_front_deque(S_TASKQUEUE * deque, S_TASK * elem){
    def_err_handler(!deque,"thqueue_deque deque", ERR_NULL);

    if(deque->max == deque->size){
        err_code failure = task_queue_realloc(deque);
        def_err_handler(failure, "add_front_deque", failure);
    }

    deque->start--; 
    if(deque->start < 0 ) deque->start += deque->max  ;

    deque->tasks[deque->start] = *elem; 
    deque->size++;

    return ERR_OK;
}//ok

err_code pop_back_deque(S_TASKQUEUE * deque, S_TASK * elem){
    def_err_handler(!deque,"pop_back_deque", ERR_NULL);
    def_err_handler(!deque,"pop_back_deque", ERR_NULL);
    warning_handler(deque->size == 0, "pop_back_deque",ERR_VAL, elem = NULL ; return ERR_OK;);

    int64_t back_idx =  (deque->start + deque->size - 1 ) % deque->max;
    if(! (deque->tasks[back_idx].function)){
        //this error is NOT recoverable so you die
        def_err_handler(ERR_VAL, "pop_back_deque", ERR_VAL);
        exit(ERR_VAL);
    }
    
    *elem = deque->tasks[back_idx];
    deque->size--;

    return ERR_OK;
}//works

void task_queue_destroy(S_TASKQUEUE * queue){
    if(queue){
        if(queue->tasks){
            free(queue->tasks);
        }
    }
}//tested ; works

bool task_queue_empty(S_TASKQUEUE * queue){
    return queue->size == 0 ;
}//tested ; inneficent ; should be a macro or smg

#ifdef debug
void dump_deque(S_TASKQUEUE * queue){
    for(uint32_t i = 0 ; i < queue->max ; i++){
        printf("i=%u , q.task[i].fn=%p, q.task[i].args=%p\n",i, (void*)queue->tasks[i].function, (void*)queue->tasks[i].args);
    }
}
#endif
