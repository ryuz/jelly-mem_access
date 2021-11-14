// ---------------------------------------------------------------------------
//  Jelly  -- The platform for real-time computing
//
//                                 Copyright (C) 2008-2020 by Ryuz
//                                 https://github.com/ryuz/jelly.git
// ---------------------------------------------------------------------------



`timescale 1ns / 1ps
`default_nettype none


module jelly_priority_queue
        #(
            parameter int   N          = 16,
            parameter int   ID_WIDTH   = 4,
            parameter int   PRI_WIDTH  = 4,
            parameter int   N_WIDTH    = $clog2(N+1)
        )
        (
            input   wire                            reset,
            input   wire                            clk,
            input   wire                            cke,

            input   wire    [0:0]                   in_op,  // 0: add, 1: del
            input   wire    [ID_WIDTH-1:0]          in_id,
            input   wire    [PRI_WIDTH-1:0]         in_pri,
            input   wire                            in_valid,

            output  wire    [ID_WIDTH-1:0]          top_id,
            output  wire    [PRI_WIDTH-1:0]         top_pri,
            output  wire                            top_valid,

            output  wire    [N_WIDTH-1:0]           size
        );


    
    typedef struct packed {
        logic                    move;
        logic                    sort;
        logic   [ID_WIDTH-1:0]   id;
        logic   [PRI_WIDTH-1:0]  pri;
        logic                    valid;
    } object_t;
    

    logic       [N_WIDTH-1:0]   reg_size;
    object_t    [N-1:0]         reg_array;
    object_t    [N:0]           array;
    always_comb begin : blk_array
        automatic object_t tmp_obj = 'x;

        array[N-1:0] = reg_array;
        array[N]     = '0;
        for ( int i = 0; i < N; ++i ) begin
            if ( array[i].move ) begin
                array[i] = array[i+1];
            end
        end

        for ( int i = 0; i < N; ++i ) begin
            if ( array[i].sort && array[i+1].valid && array[i].pri >= array[i+1].pri ) begin
                tmp_obj    = array[i+1];
                array[i+1] = array[i];
                array[i]   = tmp_obj;
            end
            else begin
                array[i].sort = 1'b0;
            end
        end
    end

    always_ff @(posedge clk) begin
        if ( reset ) begin
            reg_size  <= '0;
            for ( int i = 0; i < N; ++i ) begin
                reg_array[i].move  <= 1'b0;
                reg_array[i].sort  <= 1'b0;
                reg_array[i].id    <= 'x;
                reg_array[i].pri   <= 'x;
                reg_array[i].valid <= 1'b0;
            end
        end
        else if ( cke ) begin
            reg_array[N-1:0] <= array[N-1:0];

            if ( in_valid ) begin
                if (in_op == 1'b0) begin    // add
                    automatic   object_t    obj;

                    reg_size <= reg_size + 1'b1;
                    for ( int i = N-1; i > 1; --i ) begin
                        reg_array[i] <= reg_array[i-1];
                    end

                    obj.move  = 1'b0;
                    obj.sort  = 1'b1;
                    obj.id    = in_id;
                    obj.pri   = in_pri;
                    obj.valid = 1'b1;
                    if ( reg_array[0].valid && obj.pri >= reg_array[0].pri ) begin
                        reg_array[1] <= obj;
                    end
                    else begin
                        obj.sort = 1'b0;
                        reg_array[1] <= reg_array[0];
                        reg_array[0] <= obj;
                    end
                end
                else if ( in_op == 1'b1 ) begin    // del
                    automatic bit   del_flag;

                    reg_size <= reg_size - 1'b1;
                    del_flag = 1'b0;
                    for ( int i = 0; i < N; ++i ) begin
                        if ( array[i].id == in_id ) begin
                            del_flag = 1'b1;
                            reg_array[i].move  <= 1'b0;
                            reg_array[i].sort  <= 1'b0;
                            reg_array[i].id    <= 'x;
                            reg_array[i].pri   <= 'x;
                            reg_array[i].valid <= 1'b0;
                        end

                        if ( del_flag ) begin
                            if ( i < 2 ) begin
                                reg_array[i] <= array[i+1];
                            end
                            else begin
                                reg_array[i].move <= 1'b1;
                            end
                        end
                    end
                end
            end
        end
    end

    assign top_id    = array[0].id;
    assign top_pri   = array[0].pri;
    assign top_valid = array[0].valid;
    assign size      = reg_size;

endmodule


`default_nettype wire


// End of file
