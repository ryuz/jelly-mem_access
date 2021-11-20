// ---------------------------------------------------------------------------
//  Jelly  -- The platform for real-time computing
//
//                                 Copyright (C) 2008-2021 by Ryuz
//                                 https://github.com/ryuz/jelly.git
// ---------------------------------------------------------------------------



`timescale 1ns / 1ps
`default_nettype none


module jelly_rtos_task
        #(
            parameter   int                         TSKID_WIDTH  = 4,
            parameter   int                         TSKPRI_WIDTH = 4,
            parameter   int                         SEMID_WIDTH  = 4,
            parameter   int                         FLGPTN_WIDTH = 4,
            parameter   bit     [TSKID_WIDTH-1:0]   TSKID        = 0
        )
        (
            input   wire                        reset,
            input   wire                        clk,
            input   wire                        cke,

            input   wire                        rdy_tsk,
            input   wire                        wup_tsk,
            input   wire                        slp_tsk,
            input   wire                        rel_wai,
            
            output  wire    [TSKPRI_WIDTH-1:0]  tskpri,
            output  reg                         req_rdq,

            input   wire    [FLGPTN_WIDTH-1:0]  evtflg_flgptn,
            input   wire    [0:0]               wai_flg_wfmode,
            input   wire    [FLGPTN_WIDTH-1:0]  wai_flg_flgptn,
            input   wire                        wai_flg
            
            // monitoring
            /*
            input   wire    [TSKID_WIDTH-1:0]   rdq_add_tskid,
            input   wire                        rdq_add_valid,

            input   wire    [TSKID_WIDTH-1:0]   rdq_rmv_tskid,
            input   wire                        rdq_rmv_valid,

            input   wire    [TSKID_WIDTH-1:0]   relwai_tskid,
            input   wire                        relwai_valid,

            input   wire    [SEMID_WIDTH-1:0]   sem_wait_semid,
            input   wire    [TSKID_WIDTH-1:0]   sem_wait_tskid,
            input   wire                        sem_wait_valid
            */
        );


    typedef enum {
        TS_SLEEP  = 0,
        TS_REQRDY = 1,
        TS_READY  = 2,
        TS_WAISEM = 3,
        TS_WAIFLG = 4
    } task_status_t;

    /*
    wire    task_remove = (rdq_rmv_valid  && (rdq_rmv_tskid == TSKID));
    wire    task_ready  = (rdq_add_valid  && (rdq_add_tskid == TSKID));
    wire    task_waisem = 1'b0;//(sem_wait_valid && (rdq_add_tskid == TSKID));
    wire    task_relwai = rel_wai; //(rel_wai_valid   && (rel_wai_tskid == TSKID));
    wire    task_wakeup = wup_tsk;
    wire    task_nop    = (!task_remove && !task_ready && !task_waisem && task_relwai && !task_wakeup);
    */

    task_status_t   status, next_status;

    always_comb begin : blk_status
        next_status = status;

        unique0 case ( 1'b1 )
        rdy_tsk:        begin   next_status = TS_READY;    end
        wup_tsk:        begin   next_status = TS_REQRDY;   end
        slp_tsk:        begin   next_status = TS_SLEEP;    end
        rel_wai:        begin   next_status = TS_REQRDY;   end
//      wai_sem:        begin   next_status = TS_WAISEM;   end
        endcase
    end

    assign tskpri = TSKID;

    always_ff @(posedge clk) begin
        if ( reset ) begin
            status  <= TS_SLEEP;
            req_rdq <= 1'b0;
        end
        else if ( cke ) begin
            status  <= next_status;
            req_rdq <= (next_status == TS_REQRDY);
        end
    end

endmodule


`default_nettype wire


// End of file
