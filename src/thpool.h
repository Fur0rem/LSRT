#ifndef THC_H 
#define THC_H 

#include "common.h"
#include "job_deque.h"
#include <bits/pthreadtypes.h>
#include <stdint.h>

typedef struct thread_tab{
    uint32_t size ;
    pthread_t * threads ;
}S_THTAB;

extern uint32_t global_submit;
typedef struct s_th_pool{

    S_TASKQUEUE task_queue ;
    S_THTAB thtab ;

    pthread_mutex_t mutex_queue;
    pthread_cond_t cond_queue;

    //atomic + mutex is unnecessary I think
    //and the program breaks if I remove the atomic
    volatile _Atomic uint32_t nb_submitted ; 
    volatile _Atomic uint32_t nb_completed ;
    pthread_mutex_t mutex_sub_equals_comp ; 
    pthread_cond_t cond_sub_equals_comp ;

    volatile  uint32_t nb_th_working ; //unused atm
    volatile  uint16_t flags ; 
    /*
    1<<0 -> initialized
    1<<1 -> accepts new tasks
    1<<2 -> shutdown
    */
  
}S_THPOOL ; 


extern err_code thpool_init(S_THPOOL * pool );
/*
    pool -> not null & not initialized ; 
    initializes the thread pool with a default queue size
    and a default number of threads ; 
    starts the thread
*/

extern err_code thpool_start(S_THPOOL * pool ) ;
/*
    pool -> not null & initialized ;
    starts every thread in the pool 
*/

extern err_code thpool_append_task(S_THPOOL * pool , S_TASK * task);
/*
    pool -> not null & initialized ; 
    task -> not null ; 
    appends the task to the thpool queue
*/


extern err_code thpool_append_task_batch(S_THPOOL * pool , S_TASK * tasks, uint32_t nb_tasks);
/*
    pool -> not null & initialized ; 
    tasks -> not null ; 
    appends the tasks to the thpool queue

*/

extern err_code thpool_wait_for_all(S_THPOOL * pool);
/*
    pool -> not null & initialized ;
    waits for all the threads to finish their tasks
*/

extern err_code thpool_restart(S_THPOOL * pool); 
/*
    pool -> not null & initialized ;
    restarts the pool 
    ie : starts to accept new tasks again

    kinda hacky atm, bc it waits 
    for the tasks to finish before restarting
    it prolly should wait for the threads 
    to be idle or smtg
*/

extern err_code thpool_destroy(S_THPOOL * pool );
/*
    pool -> not null ; 
    shutdowns the pool's thread,
    destroys the thread pool, frees it's memory
    and frees the queue 

    mask can be : 
    
    1<<1 -> shutdown nice -> wait for all running threads to finish and stop
    1<<2 -> shutdown force -> force stops threads 
    1<<3 -> shutdown clean -> waits for the queue to be empty and destroys the pool
    (shutdown clean not implemented atm&)

*/


#endif 
