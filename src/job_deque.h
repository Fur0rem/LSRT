#ifndef JOB_DEQUE_H
#define JOB_DEQUE_H

#include "common.h"

typedef struct s_task{
    void (* function)(void*); 
    void * args ;
}S_TASK;
/*
    simple wrapper for 
    function pointer with it's 
    args to pass to 
*/

typedef struct s_queue{
    volatile _Atomic int64_t max ; 
    volatile _Atomic int64_t size ; 

    volatile _Atomic int64_t start ; 

    S_TASK * tasks ; 
}S_TASKQUEUE ; 
/*
    simple dequeue for tasks
    implemented with a dynamic array. 
    Maybe a linked list would have 
    been better but have strong 
    opinions about linked structures.
*/

extern err_code task_queue_init(S_TASKQUEUE * queue);
/*
    queue -> not null & not initialized ; 
    initializes the queue with a max size of 
    default queue 
*/

extern err_code add_front_deque(S_TASKQUEUE * deque, S_TASK * elem);
#define task_queue_push add_front_deque
/*
    queue -> not null & initialized ; 
    task -> not null ; 
    pushes the task into the queue
    if the queue is full, reallocates it. 
*/

extern err_code pop_back_deque(S_TASKQUEUE * deque, S_TASK * elem); 
#define task_queue_pop  pop_back_deque
/*
    queue -> not null & initialized ; 
    task -> not null ; 
    pops the task from the queue
    if the queue is empty, returns a NULL pointer
*/

extern void task_queue_destroy(S_TASKQUEUE * queue);
/*
    queue -> not null ; 
    destroys the queue and frees the memory
    allocated to it. 
*/

extern bool task_queue_empty(S_TASKQUEUE * queue);
/*
    queue -> not null & initialized ; 
    returns true if the queue is empty
    false otherwise

    doesn't check for error ;
    tbf it could be a macro

    not used ; might remove
*/

#ifdef debug 
extern void task_queue_print(S_TASKQUEUE * queue);
#endif

#endif 
