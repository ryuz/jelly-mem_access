// ---------------------------------------------------------------------------
//  Jelly  -- the system on fpga system
//
//  Ultra96V2 Real-Time OS
//
//                                 Copyright (C) 2008-2021 by Ryuz
//                                 https://github.com/ryuz/jelly.git
// ---------------------------------------------------------------------------


`timescale 1ns / 1ps
`default_nettype none


module ultra96v2_rtos
            (
                output  wire    [1:0]   led
            );
    
    
    
    // -----------------------------
    //  ZynqMP PS
    // -----------------------------
    
    localparam  AXI4L_ADDR_WIDTH = 29;
    localparam  AXI4L_DATA_SIZE  = 2;
    localparam  AXI4L_DATA_WIDTH = (8 << AXI4L_DATA_SIZE);
    localparam  AXI4L_STRB_WIDTH = AXI4L_DATA_WIDTH / 8;
    
    logic                           axi4l_aresetn;
    logic                           axi4l_aclk;
    logic   [AXI4L_ADDR_WIDTH-1:0]  axi4l_awaddr;
    logic   [2:0]                   axi4l_awprot;
    logic                           axi4l_awvalid;
    logic                           axi4l_awready;
    logic   [AXI4L_DATA_WIDTH-1:0]  axi4l_wdata;
    logic   [AXI4L_STRB_WIDTH-1:0]  axi4l_wstrb;
    logic                           axi4l_wvalid;
    logic                           axi4l_wready;
    logic   [1:0]                   axi4l_bresp;
    logic                           axi4l_bvalid;
    logic                           axi4l_bready;
    logic   [AXI4L_ADDR_WIDTH-1:0]  axi4l_araddr;
    logic   [2:0]                   axi4l_arprot;
    logic                           axi4l_arvalid;
    logic                           axi4l_arready;
    logic   [AXI4L_DATA_WIDTH-1:0]  axi4l_rdata;
    logic   [1:0]                   axi4l_rresp;
    logic                           axi4l_rvalid;
    logic                           axi4l_rready;
    
    logic   [0:0]                   irq_rtos;
    
    design_1
        i_design_1
            (
                .m_axi4l_aresetn    (axi4l_aresetn),
                .m_axi4l_aclk       (axi4l_aclk),
                .m_axi4l_awaddr     (axi4l_awaddr),
                .m_axi4l_awprot     (axi4l_awprot),
                .m_axi4l_awvalid    (axi4l_awvalid),
                .m_axi4l_awready    (axi4l_awready),
                .m_axi4l_wdata      (axi4l_wdata),
                .m_axi4l_wstrb      (axi4l_wstrb),
                .m_axi4l_wvalid     (axi4l_wvalid),
                .m_axi4l_wready     (axi4l_wready),
                .m_axi4l_bresp      (axi4l_bresp),
                .m_axi4l_bvalid     (axi4l_bvalid),
                .m_axi4l_bready     (axi4l_bready),
                .m_axi4l_araddr     (axi4l_araddr),
                .m_axi4l_arprot     (axi4l_arprot),
                .m_axi4l_arvalid    (axi4l_arvalid),
                .m_axi4l_arready    (axi4l_arready),
                .m_axi4l_rdata      (axi4l_rdata),
                .m_axi4l_rresp      (axi4l_rresp),
                .m_axi4l_rvalid     (axi4l_rvalid),
                .m_axi4l_rready     (axi4l_rready),
                
                .nfiq0_lpd_rpu      (1'b1),
                .nirq0_lpd_rpu      (~irq_rtos),
                .nfiq1_lpd_rpu      (1'b1),
                .nirq1_lpd_rpu      (1'b1)
            );
    
    
    // -----------------------------
    //  Peripheral BUS (WISHBONE)
    // -----------------------------
    
    localparam  WB_DAT_SIZE  = AXI4L_DATA_SIZE;
    localparam  WB_ADR_WIDTH = AXI4L_ADDR_WIDTH - WB_DAT_SIZE;
    localparam  WB_DAT_WIDTH = (8 << WB_DAT_SIZE);
    localparam  WB_SEL_WIDTH = (1 << WB_DAT_SIZE);
    
    logic                           reset;
    logic                           clk;
    
    logic   [WB_ADR_WIDTH-1:0]      wb_adr_i;
    logic   [WB_DAT_WIDTH-1:0]      wb_dat_i;
    logic   [WB_DAT_WIDTH-1:0]      wb_dat_o;
    logic                           wb_we_i;
    logic   [WB_SEL_WIDTH-1:0]      wb_sel_i;
    logic                           wb_stb_i;
    logic                           wb_ack_o;
    
    jelly_axi4l_to_wishbone
            #(
                .AXI4L_ADDR_WIDTH   (AXI4L_ADDR_WIDTH),
                .AXI4L_DATA_SIZE    (AXI4L_DATA_SIZE)     // 0:8bit, 1:16bit, 2:32bit, 3:64bit, ...
            )
        i_axi4l_to_wishbone
            (
                .s_axi4l_aresetn    (axi4l_aresetn),
                .s_axi4l_aclk       (axi4l_aclk),
                .s_axi4l_awaddr     (axi4l_awaddr),
                .s_axi4l_awprot     (axi4l_awprot),
                .s_axi4l_awvalid    (axi4l_awvalid),
                .s_axi4l_awready    (axi4l_awready),
                .s_axi4l_wstrb      (axi4l_wstrb),
                .s_axi4l_wdata      (axi4l_wdata),
                .s_axi4l_wvalid     (axi4l_wvalid),
                .s_axi4l_wready     (axi4l_wready),
                .s_axi4l_bresp      (axi4l_bresp),
                .s_axi4l_bvalid     (axi4l_bvalid),
                .s_axi4l_bready     (axi4l_bready),
                .s_axi4l_araddr     (axi4l_araddr),
                .s_axi4l_arprot     (axi4l_arprot),
                .s_axi4l_arvalid    (axi4l_arvalid),
                .s_axi4l_arready    (axi4l_arready),
                .s_axi4l_rdata      (axi4l_rdata),
                .s_axi4l_rresp      (axi4l_rresp),
                .s_axi4l_rvalid     (axi4l_rvalid),
                .s_axi4l_rready     (axi4l_rready),
                
                .m_wb_rst_o         (reset),
                .m_wb_clk_o         (clk),
                .m_wb_adr_o         (wb_adr_i),
                .m_wb_dat_o         (wb_dat_i),
                .m_wb_dat_i         (wb_dat_o),
                .m_wb_we_o          (wb_we_i),
                .m_wb_sel_o         (wb_sel_i),
                .m_wb_stb_o         (wb_stb_i),
                .m_wb_ack_i         (wb_ack_o)
            );
    
    
    // -----------------------------
    //  RTOS
    // -----------------------------
    
    wire    [WB_DAT_WIDTH-1:0]      wb_rtos_dat_o;
    wire                            wb_rtos_stb_i;
    wire                            wb_rtos_ack_o;
    
    jelly_rtos
            #(
                .WB_ADR_WIDTH       (WB_ADR_WIDTH),
                .WB_DAT_WIDTH       (WB_DAT_WIDTH),
                .TASKS              (16),
                .SEMAPHORES         (8),
                .TSKPRI_WIDTH       (4),
                .SEMCNT_WIDTH       (4),
                .FLGPTN_WIDTH       (32),
                .SYSTIM_WIDTH       (64),
                .RELTIM_WIDTH       (32),
                .INIT_FLGPTN        ('0)
            )
        i_rtos
            (
                .reset              (reset),
                .clk                (clk),
                .cke                (1'b1),
                
                .s_wb_adr_i         (wb_adr_i),
                .s_wb_dat_i         (wb_dat_i),
                .s_wb_dat_o         (wb_rtos_dat_o),
                .s_wb_we_i          (wb_we_i ),
                .s_wb_sel_i         (wb_sel_i),
                .s_wb_stb_i         (wb_rtos_stb_i),
                .s_wb_ack_o         (wb_rtos_ack_o),
                
                .irq                (irq_rtos)
            );
    
    
    
    // -----------------------------
    //  Test LED
    // -----------------------------
    
    wire    [WB_DAT_WIDTH-1:0]      wb_led_dat_o;
    wire                            wb_led_stb_i;
    wire                            wb_led_ack_o;
    
    reg     [0:0]                   reg_led;
    always_ff @(posedge clk) begin
        if ( reset ) begin
            reg_led <= 0;
        end
        else begin
            if (wb_led_stb_i && wb_we_i && wb_sel_i[0]) begin
                reg_led <= wb_dat_i[0:0];
            end
        end
    end
    
    assign wb_led_dat_o = reg_led;
    assign wb_led_ack_o = wb_led_stb_i;
    
    
    reg     [25:0]  reg_clk_count;
    always_ff @(posedge clk) begin
        if ( reset ) begin
            reg_clk_count <= 0;
        end
        else begin
            reg_clk_count <= reg_clk_count + 1;
        end
    end
    
    assign led[0] = reg_led;
    assign led[1] = reg_clk_count[25];
    
    
    
    
    // -----------------------------
    //  Test Timer
    // -----------------------------
    
    wire    [WB_DAT_WIDTH-1:0]      wb_tim_dat_o;
    wire                            wb_tim_stb_i;
    wire                            wb_tim_ack_o;
    
    jelly_interval_timer
            #(
                .WB_ADR_WIDTH       (2),
                .WB_DAT_WIDTH       (WB_DAT_WIDTH),
                .IRQ_LEVEL          (1)
            )
        i_interval_timer
            (
                .reset              (reset),
                .clk                (clk),
                
                .interrupt_req      (),
                
                .s_wb_adr_i         (wb_adr_i[1:0]),
                .s_wb_dat_o         (wb_tim_dat_o),
                .s_wb_dat_i         (wb_dat_i),
                .s_wb_we_i          (wb_we_i),
                .s_wb_sel_i         (wb_sel_i),
                .s_wb_stb_i         (wb_tim_stb_i),
                .s_wb_ack_o         (wb_tim_ack_o)
            );
    
    
    
    // -----------------------------
    //  WISHBONE address decode
    // -----------------------------
    
    assign wb_rtos_stb_i = wb_stb_i & (wb_adr_i[23:16] == 8'h00);
    assign wb_led_stb_i  = wb_stb_i & (wb_adr_i[23:16] == 8'h01);
    assign wb_tim_stb_i  = wb_stb_i & (wb_adr_i[23:16] == 8'h02);
    
    assign wb_dat_o      = wb_rtos_stb_i ? wb_rtos_dat_o :
                           wb_led_stb_i  ? wb_led_dat_o  :
                           wb_tim_stb_i  ? wb_tim_dat_o  :
                           {WB_DAT_WIDTH{1'b0}};
    
    assign wb_ack_o      = wb_rtos_stb_i ? wb_rtos_ack_o :
                           wb_led_stb_i  ? wb_led_ack_o  :
                           wb_tim_stb_i  ? wb_tim_ack_o  :
                           wb_stb_i;
    
    
endmodule



`default_nettype wire


// end of file
