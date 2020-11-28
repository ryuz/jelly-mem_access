// ---------------------------------------------------------------------------
//  Jelly  -- The FPGA processing system
//
//  IMX219 capture sample
//
//                                 Copyright (C) 2008-2020 by Ryuz
//                                 https://github.com/ryuz/
// ---------------------------------------------------------------------------




`timescale 1ns / 1ps
`default_nettype none


module ultra96v2_imx219_stepper_motor
        #(
            parameter   X_NUM = 3280,
            parameter   Y_NUM = 2464
        )
        (
            output  wire    [1:0]   radio_led,
            
            input   wire    [1:0]   push_sw,
            input   wire    [3:0]   dip_sw,
            output  wire    [3:0]   led,
            output  wire    [7:0]   pmod0,
            output  wire    [7:0]   pmod1,
            
            input   wire            cam_clk_p,
            input   wire            cam_clk_n,
            input   wire    [1:0]   cam_data_p,
            input   wire    [1:0]   cam_data_n
        );
    
    
    wire            sys_reset;
    wire            sys_clk100;
    wire            sys_clk200;
    wire            sys_clk250;
    
    localparam  AXI4L_PERI_ADDR_WIDTH = 40;
    localparam  AXI4L_PERI_DATA_SIZE  = 3;     // 0:8bit, 1:16bit, 2:32bit, 3:64bit ...
    localparam  AXI4L_PERI_DATA_WIDTH = (8 << AXI4L_PERI_DATA_SIZE);
    localparam  AXI4L_PERI_STRB_WIDTH = AXI4L_PERI_DATA_WIDTH / 8;
    
    wire                                 axi4l_peri_aresetn;
    wire                                 axi4l_peri_aclk;
    wire    [AXI4L_PERI_ADDR_WIDTH-1:0]  axi4l_peri_awaddr;
    wire    [2:0]                        axi4l_peri_awprot;
    wire                                 axi4l_peri_awvalid;
    wire                                 axi4l_peri_awready;
    wire    [AXI4L_PERI_STRB_WIDTH-1:0]  axi4l_peri_wstrb;
    wire    [AXI4L_PERI_DATA_WIDTH-1:0]  axi4l_peri_wdata;
    wire                                 axi4l_peri_wvalid;
    wire                                 axi4l_peri_wready;
    wire    [1:0]                        axi4l_peri_bresp;
    wire                                 axi4l_peri_bvalid;
    wire                                 axi4l_peri_bready;
    wire    [AXI4L_PERI_ADDR_WIDTH-1:0]  axi4l_peri_araddr;
    wire    [2:0]                        axi4l_peri_arprot;
    wire                                 axi4l_peri_arvalid;
    wire                                 axi4l_peri_arready;
    wire    [AXI4L_PERI_DATA_WIDTH-1:0]  axi4l_peri_rdata;
    wire    [1:0]                        axi4l_peri_rresp;
    wire                                 axi4l_peri_rvalid;
    wire                                 axi4l_peri_rready;
    
    
    
    localparam  AXI4_MEM0_ID_WIDTH   = 6;
    localparam  AXI4_MEM0_ADDR_WIDTH = 49;
    localparam  AXI4_MEM0_DATA_SIZE  = 4;   // 2:32bit, 3:64bit, 4:128bit
    localparam  AXI4_MEM0_DATA_WIDTH = (8 << AXI4_MEM0_DATA_SIZE);
    localparam  AXI4_MEM0_STRB_WIDTH = AXI4_MEM0_DATA_WIDTH / 8;
    
    wire                                 axi4_mem_aresetn;
    wire                                 axi4_mem_aclk;
    
    wire    [AXI4_MEM0_ID_WIDTH-1:0]     axi4_mem0_awid;
    wire    [AXI4_MEM0_ADDR_WIDTH-1:0]   axi4_mem0_awaddr;
    wire    [1:0]                        axi4_mem0_awburst;
    wire    [3:0]                        axi4_mem0_awcache;
    wire    [7:0]                        axi4_mem0_awlen;
    wire    [0:0]                        axi4_mem0_awlock;
    wire    [2:0]                        axi4_mem0_awprot;
    wire    [3:0]                        axi4_mem0_awqos;
    wire    [3:0]                        axi4_mem0_awregion;
    wire    [2:0]                        axi4_mem0_awsize;
    wire                                 axi4_mem0_awvalid;
    wire                                 axi4_mem0_awready;
    wire    [AXI4_MEM0_STRB_WIDTH-1:0]   axi4_mem0_wstrb;
    wire    [AXI4_MEM0_DATA_WIDTH-1:0]   axi4_mem0_wdata;
    wire                                 axi4_mem0_wlast;
    wire                                 axi4_mem0_wvalid;
    wire                                 axi4_mem0_wready;
    wire    [AXI4_MEM0_ID_WIDTH-1:0]     axi4_mem0_bid;
    wire    [1:0]                        axi4_mem0_bresp;
    wire                                 axi4_mem0_bvalid;
    wire                                 axi4_mem0_bready;
    wire    [AXI4_MEM0_ID_WIDTH-1:0]     axi4_mem0_arid;
    wire    [AXI4_MEM0_ADDR_WIDTH-1:0]   axi4_mem0_araddr;
    wire    [1:0]                        axi4_mem0_arburst;
    wire    [3:0]                        axi4_mem0_arcache;
    wire    [7:0]                        axi4_mem0_arlen;
    wire    [0:0]                        axi4_mem0_arlock;
    wire    [2:0]                        axi4_mem0_arprot;
    wire    [3:0]                        axi4_mem0_arqos;
    wire    [3:0]                        axi4_mem0_arregion;
    wire    [2:0]                        axi4_mem0_arsize;
    wire                                 axi4_mem0_arvalid;
    wire                                 axi4_mem0_arready;
    wire    [AXI4_MEM0_ID_WIDTH-1:0]     axi4_mem0_rid;
    wire    [1:0]                        axi4_mem0_rresp;
    wire    [AXI4_MEM0_DATA_WIDTH-1:0]   axi4_mem0_rdata;
    wire                                 axi4_mem0_rlast;
    wire                                 axi4_mem0_rvalid;
    wire                                 axi4_mem0_rready;
    
    design_1
        i_design_1
            (
                .out_reset              (sys_reset),
                .out_clk100             (sys_clk100),
                .out_clk200             (sys_clk200),
                .out_clk250             (sys_clk250),
                
                .m_axi4l_peri_aresetn   (axi4l_peri_aresetn),
                .m_axi4l_peri_aclk      (axi4l_peri_aclk),
                .m_axi4l_peri_awaddr    (axi4l_peri_awaddr),
                .m_axi4l_peri_awprot    (axi4l_peri_awprot),
                .m_axi4l_peri_awvalid   (axi4l_peri_awvalid),
                .m_axi4l_peri_awready   (axi4l_peri_awready),
                .m_axi4l_peri_wstrb     (axi4l_peri_wstrb),
                .m_axi4l_peri_wdata     (axi4l_peri_wdata),
                .m_axi4l_peri_wvalid    (axi4l_peri_wvalid),
                .m_axi4l_peri_wready    (axi4l_peri_wready),
                .m_axi4l_peri_bresp     (axi4l_peri_bresp),
                .m_axi4l_peri_bvalid    (axi4l_peri_bvalid),
                .m_axi4l_peri_bready    (axi4l_peri_bready),
                .m_axi4l_peri_araddr    (axi4l_peri_araddr),
                .m_axi4l_peri_arprot    (axi4l_peri_arprot),
                .m_axi4l_peri_arvalid   (axi4l_peri_arvalid),
                .m_axi4l_peri_arready   (axi4l_peri_arready),
                .m_axi4l_peri_rdata     (axi4l_peri_rdata),
                .m_axi4l_peri_rresp     (axi4l_peri_rresp),
                .m_axi4l_peri_rvalid    (axi4l_peri_rvalid),
                .m_axi4l_peri_rready    (axi4l_peri_rready),
                
                
                .s_axi4_mem_aresetn     (axi4_mem_aresetn),
                .s_axi4_mem_aclk        (axi4_mem_aclk),
                
                .s_axi4_mem0_awid       (axi4_mem0_awid),
                .s_axi4_mem0_awaddr     (axi4_mem0_awaddr),
                .s_axi4_mem0_awburst    (axi4_mem0_awburst),
                .s_axi4_mem0_awcache    (axi4_mem0_awcache),
                .s_axi4_mem0_awlen      (axi4_mem0_awlen),
                .s_axi4_mem0_awlock     (axi4_mem0_awlock),
                .s_axi4_mem0_awprot     (axi4_mem0_awprot),
                .s_axi4_mem0_awqos      (axi4_mem0_awqos),
    //          .s_axi4_mem0_awregion   (axi4_mem0_awregion),
                .s_axi4_mem0_awsize     (axi4_mem0_awsize),
                .s_axi4_mem0_awvalid    (axi4_mem0_awvalid),
                .s_axi4_mem0_awready    (axi4_mem0_awready),
                .s_axi4_mem0_wstrb      (axi4_mem0_wstrb),
                .s_axi4_mem0_wdata      (axi4_mem0_wdata),
                .s_axi4_mem0_wlast      (axi4_mem0_wlast),
                .s_axi4_mem0_wvalid     (axi4_mem0_wvalid),
                .s_axi4_mem0_wready     (axi4_mem0_wready),
                .s_axi4_mem0_bid        (axi4_mem0_bid),
                .s_axi4_mem0_bresp      (axi4_mem0_bresp),
                .s_axi4_mem0_bvalid     (axi4_mem0_bvalid),
                .s_axi4_mem0_bready     (axi4_mem0_bready),
                .s_axi4_mem0_araddr     (axi4_mem0_araddr),
                .s_axi4_mem0_arburst    (axi4_mem0_arburst),
                .s_axi4_mem0_arcache    (axi4_mem0_arcache),
                .s_axi4_mem0_arid       (axi4_mem0_arid),
                .s_axi4_mem0_arlen      (axi4_mem0_arlen),
                .s_axi4_mem0_arlock     (axi4_mem0_arlock),
                .s_axi4_mem0_arprot     (axi4_mem0_arprot),
                .s_axi4_mem0_arqos      (axi4_mem0_arqos),
    //          .s_axi4_mem0_arregion   (axi4_mem0_arregion),
                .s_axi4_mem0_arsize     (axi4_mem0_arsize),
                .s_axi4_mem0_arvalid    (axi4_mem0_arvalid),
                .s_axi4_mem0_arready    (axi4_mem0_arready),
                .s_axi4_mem0_rid        (axi4_mem0_rid),
                .s_axi4_mem0_rresp      (axi4_mem0_rresp),
                .s_axi4_mem0_rdata      (axi4_mem0_rdata),
                .s_axi4_mem0_rlast      (axi4_mem0_rlast),
                .s_axi4_mem0_rvalid     (axi4_mem0_rvalid),
                .s_axi4_mem0_rready     (axi4_mem0_rready)
            );
    
    
    
    // AXI4L => WISHBONE
    localparam  WB_ADR_WIDTH = AXI4L_PERI_ADDR_WIDTH - AXI4L_PERI_DATA_SIZE;
    localparam  WB_DAT_SIZE  = AXI4L_PERI_DATA_SIZE;
    localparam  WB_DAT_WIDTH = AXI4L_PERI_DATA_WIDTH;
    localparam  WB_SEL_WIDTH = AXI4L_PERI_STRB_WIDTH;
    
    wire                           wb_peri_rst_i;
    wire                           wb_peri_clk_i;
    wire    [WB_ADR_WIDTH-1:0]     wb_peri_adr_i;
    wire    [WB_DAT_WIDTH-1:0]     wb_peri_dat_o;
    wire    [WB_DAT_WIDTH-1:0]     wb_peri_dat_i;
    wire    [WB_SEL_WIDTH-1:0]     wb_peri_sel_i;
    wire                           wb_peri_we_i;
    wire                           wb_peri_stb_i;
    wire                           wb_peri_ack_o;
    
    jelly_axi4l_to_wishbone
            #(
                .AXI4L_ADDR_WIDTH   (AXI4L_PERI_ADDR_WIDTH),
                .AXI4L_DATA_SIZE    (AXI4L_PERI_DATA_SIZE)     // 0:8bit, 1:16bit, 2:32bit ...
            )
        i_axi4l_to_wishbone
            (
                .s_axi4l_aresetn    (axi4l_peri_aresetn),
                .s_axi4l_aclk       (axi4l_peri_aclk),
                .s_axi4l_awaddr     (axi4l_peri_awaddr),
                .s_axi4l_awprot     (axi4l_peri_awprot),
                .s_axi4l_awvalid    (axi4l_peri_awvalid),
                .s_axi4l_awready    (axi4l_peri_awready),
                .s_axi4l_wstrb      (axi4l_peri_wstrb),
                .s_axi4l_wdata      (axi4l_peri_wdata),
                .s_axi4l_wvalid     (axi4l_peri_wvalid),
                .s_axi4l_wready     (axi4l_peri_wready),
                .s_axi4l_bresp      (axi4l_peri_bresp),
                .s_axi4l_bvalid     (axi4l_peri_bvalid),
                .s_axi4l_bready     (axi4l_peri_bready),
                .s_axi4l_araddr     (axi4l_peri_araddr),
                .s_axi4l_arprot     (axi4l_peri_arprot),
                .s_axi4l_arvalid    (axi4l_peri_arvalid),
                .s_axi4l_arready    (axi4l_peri_arready),
                .s_axi4l_rdata      (axi4l_peri_rdata),
                .s_axi4l_rresp      (axi4l_peri_rresp),
                .s_axi4l_rvalid     (axi4l_peri_rvalid),
                .s_axi4l_rready     (axi4l_peri_rready),
                
                .m_wb_rst_o         (wb_peri_rst_i),
                .m_wb_clk_o         (wb_peri_clk_i),
                .m_wb_adr_o         (wb_peri_adr_i),
                .m_wb_dat_i         (wb_peri_dat_o),
                .m_wb_dat_o         (wb_peri_dat_i),
                .m_wb_sel_o         (wb_peri_sel_i),
                .m_wb_we_o          (wb_peri_we_i),
                .m_wb_stb_o         (wb_peri_stb_i),
                .m_wb_ack_i         (wb_peri_ack_o)
            );
    
    
    // ----------------------------------------
    //  Global ID
    // ----------------------------------------
    
    wire    [WB_DAT_WIDTH-1:0]  wb_gid_dat_o;
    wire                        wb_gid_stb_i;
    wire                        wb_gid_ack_o;
    
    assign wb_gid_dat_o = 32'h01234567;
    assign wb_gid_ack_o = wb_gid_stb_i;
    
    
    
    // ----------------------------------------
    //  MIPI D-PHY RX
    // ----------------------------------------
    
    (* KEEP = "true" *)
    wire                rxbyteclkhs;
    wire                clkoutphy_out;
    wire                pll_lock_out;
    wire                system_rst_out;
    wire                init_done;
    
    wire                cl_rxclkactivehs;
    wire                cl_stopstate;
    wire                cl_enable         = 1;
    wire                cl_rxulpsclknot;
    wire                cl_ulpsactivenot;
    
    wire    [7:0]       dl0_rxdatahs;
    wire                dl0_rxvalidhs;
    wire                dl0_rxactivehs;
    wire                dl0_rxsynchs;
    
    wire                dl0_forcerxmode   = 0;
    wire                dl0_stopstate;
    wire                dl0_enable        = 1;
    wire                dl0_ulpsactivenot;
    
    wire                dl0_rxclkesc;
    wire                dl0_rxlpdtesc;
    wire                dl0_rxulpsesc;
    wire    [3:0]       dl0_rxtriggeresc;
    wire    [7:0]       dl0_rxdataesc;
    wire                dl0_rxvalidesc;
    
    wire                dl0_errsoths;
    wire                dl0_errsotsynchs;
    wire                dl0_erresc;
    wire                dl0_errsyncesc;
    wire                dl0_errcontrol;
    
    wire    [7:0]       dl1_rxdatahs;
    wire                dl1_rxvalidhs;
    wire                dl1_rxactivehs;
    wire                dl1_rxsynchs;
    
    wire                dl1_forcerxmode   = 0;
    wire                dl1_stopstate;
    wire                dl1_enable        = 1;
    wire                dl1_ulpsactivenot;
    
    wire                dl1_rxclkesc;
    wire                dl1_rxlpdtesc;
    wire                dl1_rxulpsesc;
    wire    [3:0]       dl1_rxtriggeresc;
    wire    [7:0]       dl1_rxdataesc;
    wire                dl1_rxvalidesc;
    
    wire                dl1_errsoths;
    wire                dl1_errsotsynchs;
    wire                dl1_erresc;
    wire                dl1_errsyncesc;
    wire                dl1_errcontrol;
    
    mipi_dphy_cam
        i_mipi_dphy_cam
            (
                .core_clk           (sys_clk200),
                .core_rst           (sys_reset),
                .rxbyteclkhs        (rxbyteclkhs),
                
                .clkoutphy_out      (clkoutphy_out),
                .pll_lock_out       (pll_lock_out),
                .system_rst_out     (system_rst_out),
                .init_done          (init_done),
                
                .cl_rxclkactivehs   (cl_rxclkactivehs),
                .cl_stopstate       (cl_stopstate),
                .cl_enable          (cl_enable),
                .cl_rxulpsclknot    (cl_rxulpsclknot),
                .cl_ulpsactivenot   (cl_ulpsactivenot),
                
                .dl0_rxdatahs       (dl0_rxdatahs),
                .dl0_rxvalidhs      (dl0_rxvalidhs),
                .dl0_rxactivehs     (dl0_rxactivehs),
                .dl0_rxsynchs       (dl0_rxsynchs),
                
                .dl0_forcerxmode    (dl0_forcerxmode),
                .dl0_stopstate      (dl0_stopstate),
                .dl0_enable         (dl0_enable),
                .dl0_ulpsactivenot  (dl0_ulpsactivenot),
                
                .dl0_rxclkesc       (dl0_rxclkesc),
                .dl0_rxlpdtesc      (dl0_rxlpdtesc),
                .dl0_rxulpsesc      (dl0_rxulpsesc),
                .dl0_rxtriggeresc   (dl0_rxtriggeresc),
                .dl0_rxdataesc      (dl0_rxdataesc),
                .dl0_rxvalidesc     (dl0_rxvalidesc),
                
                .dl0_errsoths       (dl0_errsoths),
                .dl0_errsotsynchs   (dl0_errsotsynchs),
                .dl0_erresc         (dl0_erresc),
                .dl0_errsyncesc     (dl0_errsyncesc),
                .dl0_errcontrol     (dl0_errcontrol),
                
                .dl1_rxdatahs       (dl1_rxdatahs),
                .dl1_rxvalidhs      (dl1_rxvalidhs),
                .dl1_rxactivehs     (dl1_rxactivehs),
                .dl1_rxsynchs       (dl1_rxsynchs),
                
                .dl1_forcerxmode    (dl1_forcerxmode),
                .dl1_stopstate      (dl1_stopstate),
                .dl1_enable         (dl1_enable),
                .dl1_ulpsactivenot  (dl1_ulpsactivenot),
                
                .dl1_rxclkesc       (dl1_rxclkesc),
                .dl1_rxlpdtesc      (dl1_rxlpdtesc),
                .dl1_rxulpsesc      (dl1_rxulpsesc),
                .dl1_rxtriggeresc   (dl1_rxtriggeresc),
                .dl1_rxdataesc      (dl1_rxdataesc),
                .dl1_rxvalidesc     (dl1_rxvalidesc),
                
                .dl1_errsoths       (dl1_errsoths),
                .dl1_errsotsynchs   (dl1_errsotsynchs),
                .dl1_erresc         (dl1_erresc),
                .dl1_errsyncesc     (dl1_errsyncesc),
                .dl1_errcontrol     (dl1_errcontrol),
                
                .clk_rxp            (cam_clk_p),
                .clk_rxn            (cam_clk_n),
                .data_rxp           (cam_data_p),
                .data_rxn           (cam_data_n)
           );
    
    
    wire        dphy_clk   = rxbyteclkhs;
    wire        dphy_reset;
    jelly_reset
            #(
                .IN_LOW_ACTIVE      (0),
                .OUT_LOW_ACTIVE     (0),
                .INPUT_REGS         (2),
                .COUNTER_WIDTH      (5),
                .INSERT_BUFG        (0)
            )
        i_reset
            (
                .clk                (dphy_clk),
                .in_reset           (sys_reset || system_rst_out),
                .out_reset          (dphy_reset)
            );
    
    
    
    
    // ----------------------------------------
    //  CSI-2
    // ----------------------------------------
    
    
    wire            axi4s_cam_aresetn = ~sys_reset;
    wire            axi4s_cam_aclk    = sys_clk200;
    
    wire    [0:0]   axi4s_csi2_tuser;
    wire            axi4s_csi2_tlast;
    wire    [9:0]   axi4s_csi2_tdata;
    wire            axi4s_csi2_tvalid;
    wire            axi4s_csi2_tready;
    
    jelly_mipi_csi2_rx
            #(
                .LANES              (2),
                .DATA_WIDTH         (10),
                .M_FIFO_ASYNC       (1)
            )
        i_mipi_csi2_rx
            (
                .aresetn            (~sys_reset),
                .aclk               (sys_clk250),
                
                .rxreseths          (system_rst_out),
                .rxbyteclkhs        (dphy_clk),
                .rxdatahs           ({dl1_rxdatahs,   dl0_rxdatahs  }),
                .rxvalidhs          ({dl1_rxvalidhs,  dl0_rxvalidhs }),
                .rxactivehs         ({dl1_rxactivehs, dl0_rxactivehs}),
                .rxsynchs           ({dl1_rxsynchs,   dl0_rxsynchs  }),
                
                .m_axi4s_aresetn    (axi4s_cam_aresetn),
                .m_axi4s_aclk       (axi4s_cam_aclk),
                .m_axi4s_tuser      (axi4s_csi2_tuser),
                .m_axi4s_tlast      (axi4s_csi2_tlast),
                .m_axi4s_tdata      (axi4s_csi2_tdata),
                .m_axi4s_tvalid     (axi4s_csi2_tvalid),
                .m_axi4s_tready     (1'b1)  // (axi4s_csi2_tready)
            );
    
    
    // format regularizer
    wire    [0:0]               axi4s_fmtr_tuser;
    wire                        axi4s_fmtr_tlast;
    wire    [9:0]               axi4s_fmtr_tdata;
    wire                        axi4s_fmtr_tvalid;
    wire                        axi4s_fmtr_tready;
    
    wire    [WB_DAT_WIDTH-1:0]  wb_fmtr_dat_o;
    wire                        wb_fmtr_stb_i;
    wire                        wb_fmtr_ack_o;
    
    jelly_video_format_regularizer
            #(
                .WB_ADR_WIDTH       (8),
                .WB_DAT_WIDTH       (WB_DAT_WIDTH),
                
                .TUSER_WIDTH        (1),
                .TDATA_WIDTH        (10),
                .X_WIDTH            (16),
                .Y_WIDTH            (16),
                .TIMER_WIDTH        (32),
                .S_SLAVE_REGS       (1),
                .S_MASTER_REGS      (1),
                .M_SLAVE_REGS       (1),
                .M_MASTER_REGS      (1),
                
                .INIT_CTL_CONTROL   (2'b00),
                .INIT_CTL_SKIP      (1),
                .INIT_PARAM_WIDTH   (X_NUM),
                .INIT_PARAM_HEIGHT  (Y_NUM),
                .INIT_PARAM_FILL    (10'd0),
                .INIT_PARAM_TIMEOUT (32'h00010000)
            )
        i_video_format_regularizer
            (
                .aresetn            (axi4s_cam_aresetn),
                .aclk               (axi4s_cam_aclk),
                .aclken             (1'b1),
                
                .s_wb_rst_i         (wb_peri_rst_i),
                .s_wb_clk_i         (wb_peri_clk_i),
                .s_wb_adr_i         (wb_peri_adr_i[7:0]),
                .s_wb_dat_o         (wb_fmtr_dat_o),
                .s_wb_dat_i         (wb_peri_dat_i),
                .s_wb_we_i          (wb_peri_we_i),
                .s_wb_sel_i         (wb_peri_sel_i),
                .s_wb_stb_i         (wb_fmtr_stb_i),
                .s_wb_ack_o         (wb_fmtr_ack_o),
                
                .s_axi4s_tuser      (axi4s_csi2_tuser),
                .s_axi4s_tlast      (axi4s_csi2_tlast),
                .s_axi4s_tdata      (axi4s_csi2_tdata),
                .s_axi4s_tvalid     (axi4s_csi2_tvalid),
                .s_axi4s_tready     (axi4s_csi2_tready),
                
                .m_axi4s_tuser      (axi4s_fmtr_tuser),
                .m_axi4s_tlast      (axi4s_fmtr_tlast),
                .m_axi4s_tdata      (axi4s_fmtr_tdata),
                .m_axi4s_tvalid     (axi4s_fmtr_tvalid),
                .m_axi4s_tready     (axi4s_fmtr_tready)
            );
    
    
    // parameter update(フレーム単位で一括パラメータ更新できるように)
    wire                            parameter_update_req;
    
    wire    [WB_DAT_WIDTH-1:0]      wb_prmup_dat_o;
    wire                            wb_prmup_stb_i;
    wire                            wb_prmup_ack_o;
    
    jelly_video_parameter_update
            #(
                .WB_ADR_WIDTH       (8),
                .WB_DAT_SIZE        (WB_DAT_SIZE),
                .WB_DAT_WIDTH       (WB_DAT_WIDTH),
                
                .TUSER_WIDTH        (1),
                .TDATA_WIDTH        (10),
                
                .INDEX_WIDTH        (1),
                .FRAME_WIDTH        (32),
                
                .INIT_CONTROL       (2'b11),
                
                .DELAY              (1)
            )
        i_video_parameter_update
            (
                .aresetn            (axi4s_cam_aresetn),
                .aclk               (axi4s_cam_aclk),
                .aclken             (1'b1),
                
                .out_update_req     (parameter_update_req),
                
                .s_wb_rst_i         (wb_peri_rst_i),
                .s_wb_clk_i         (wb_peri_clk_i),
                .s_wb_adr_i         (wb_peri_adr_i[7:0]),
                .s_wb_dat_i         (wb_peri_dat_i),
                .s_wb_dat_o         (wb_prmup_dat_o),
                .s_wb_we_i          (wb_peri_we_i),
                .s_wb_sel_i         (wb_peri_sel_i),
                .s_wb_stb_i         (wb_prmup_stb_i),
                .s_wb_ack_o         (wb_prmup_ack_o),
                
                .s_axi4s_tdata      (axi4s_fmtr_tdata),
                .s_axi4s_tlast      (axi4s_fmtr_tlast),
                .s_axi4s_tuser      (axi4s_fmtr_tuser),
                .s_axi4s_tvalid     (axi4s_fmtr_tvalid),
                .s_axi4s_tready     (),
                
                .m_axi4s_tuser      (),
                .m_axi4s_tlast      (),
                .m_axi4s_tdata      (),
                .m_axi4s_tvalid     (),
                .m_axi4s_tready     (1'b1)
                
            );
    
    
    // 画像処理
    localparam  IMG_ANGLE_WIDTH = 32;
    
    wire    [0:0]                   axi4s_rgb_tuser;
    wire                            axi4s_rgb_tlast;
    wire    [39:0]                  axi4s_rgb_tdata;
    wire                            axi4s_rgb_tvalid;
    wire                            axi4s_rgb_tready;
    
    wire    [IMG_ANGLE_WIDTH-1:0]   image_angle;
    wire                            image_valid;
    
    wire    [WB_DAT_WIDTH-1:0]      wb_imgp_dat_o;
    wire                            wb_imgp_stb_i;
    wire                            wb_imgp_ack_o;
    
    image_processing
            #(
                .WB_ADR_WIDTH       (13),
                .WB_DAT_WIDTH       (WB_DAT_WIDTH),
                
                .DATA_WIDTH         (10),
                .ANGLE_WIDTH        (IMG_ANGLE_WIDTH),
                .ATAN2_X_WIDTH      (32),
                .ATAN2_Y_WIDTH      (32),
                
                .IMG_X_NUM          (640),
                .IMG_Y_NUM          (132),
                .IMG_X_WIDTH        (14),
                .IMG_Y_WIDTH        (14),
                
                .TUSER_WIDTH        (1)
            )
        i_image_processing
            (
                .aresetn            (axi4s_cam_aresetn),
                .aclk               (axi4s_cam_aclk),
                
                .in_update_req      (parameter_update_req),
                
                .s_wb_rst_i         (wb_peri_rst_i),
                .s_wb_clk_i         (wb_peri_clk_i),
                .s_wb_adr_i         (wb_peri_adr_i[12:0]),
                .s_wb_dat_o         (wb_imgp_dat_o),
                .s_wb_dat_i         (wb_peri_dat_i),
                .s_wb_we_i          (wb_peri_we_i),
                .s_wb_sel_i         (wb_peri_sel_i),
                .s_wb_stb_i         (wb_imgp_stb_i),
                .s_wb_ack_o         (wb_imgp_ack_o),
                
                .s_axi4s_tuser      (axi4s_fmtr_tuser),
                .s_axi4s_tlast      (axi4s_fmtr_tlast),
                .s_axi4s_tdata      (axi4s_fmtr_tdata),
                .s_axi4s_tvalid     (axi4s_fmtr_tvalid),
                .s_axi4s_tready     (axi4s_fmtr_tready),
                
                .m_axi4s_tuser      (axi4s_rgb_tuser),
                .m_axi4s_tlast      (axi4s_rgb_tlast),
                .m_axi4s_tdata      (axi4s_rgb_tdata),
                .m_axi4s_tvalid     (axi4s_rgb_tvalid),
                .m_axi4s_tready     (axi4s_rgb_tready),
                
                .out_reset          (wb_peri_rst_i),
                .out_clk            (wb_peri_clk_i),
                .out_angle          (image_angle),
                .out_valid          (image_valid)
            );
    
    
    // DMA write
    wire    [WB_DAT_WIDTH-1:0]  wb_vdmaw_dat_o;
    wire                        wb_vdmaw_stb_i;
    wire                        wb_vdmaw_ack_o;
    
    jelly_vdma_axi4s_to_axi4
            #(
                .ASYNC              (1),
                .FIFO_PTR_WIDTH     (12),
                
                .PIXEL_SIZE         (2),    // 32bit
                .AXI4_ID_WIDTH      (AXI4_MEM0_ID_WIDTH),
                .AXI4_ADDR_WIDTH    (AXI4_MEM0_ADDR_WIDTH),
                .AXI4_DATA_SIZE     (AXI4_MEM0_DATA_SIZE),
                .AXI4S_DATA_SIZE    (2),    // 32bit
                .AXI4S_USER_WIDTH   (1),
                .INDEX_WIDTH        (8),
                .STRIDE_WIDTH       (16),
                .H_WIDTH            (14),
                .V_WIDTH            (14),
                .SIZE_WIDTH         (32),
                
                .WB_ADR_WIDTH       (8),
                .WB_DAT_WIDTH       (WB_DAT_WIDTH),
                .INIT_CTL_CONTROL   (2'b00),
                .INIT_PARAM_ADDR    (32'h3000_0000),
                .INIT_PARAM_STRIDE  (X_NUM*2),
                .INIT_PARAM_WIDTH   (X_NUM),
                .INIT_PARAM_HEIGHT  (Y_NUM),
                .INIT_PARAM_SIZE    (X_NUM*Y_NUM),
                .INIT_PARAM_AWLEN   (7)
            )
        i_vdma_axi4s_to_axi4
            (
                .m_axi4_aresetn     (axi4_mem_aresetn),
                .m_axi4_aclk        (axi4_mem_aclk),
                .m_axi4_awid        (axi4_mem0_awid),
                .m_axi4_awaddr      (axi4_mem0_awaddr),
                .m_axi4_awburst     (axi4_mem0_awburst),
                .m_axi4_awcache     (axi4_mem0_awcache),
                .m_axi4_awlen       (axi4_mem0_awlen),
                .m_axi4_awlock      (axi4_mem0_awlock),
                .m_axi4_awprot      (axi4_mem0_awprot),
                .m_axi4_awqos       (axi4_mem0_awqos),
                .m_axi4_awregion    (),
                .m_axi4_awsize      (axi4_mem0_awsize),
                .m_axi4_awvalid     (axi4_mem0_awvalid),
                .m_axi4_awready     (axi4_mem0_awready),
                .m_axi4_wstrb       (axi4_mem0_wstrb),
                .m_axi4_wdata       (axi4_mem0_wdata),
                .m_axi4_wlast       (axi4_mem0_wlast),
                .m_axi4_wvalid      (axi4_mem0_wvalid),
                .m_axi4_wready      (axi4_mem0_wready),
                .m_axi4_bid         (axi4_mem0_bid),
                .m_axi4_bresp       (axi4_mem0_bresp),
                .m_axi4_bvalid      (axi4_mem0_bvalid),
                .m_axi4_bready      (axi4_mem0_bready),
                
                .s_axi4s_aresetn    (axi4s_cam_aresetn),
                .s_axi4s_aclk       (axi4s_cam_aclk),
                .s_axi4s_tuser      (axi4s_rgb_tuser),
                .s_axi4s_tlast      (axi4s_rgb_tlast),
                .s_axi4s_tdata      ({
                                        axi4s_rgb_tdata[39:32],
                                        axi4s_rgb_tdata[29:22],
                                        axi4s_rgb_tdata[19:12],
                                        axi4s_rgb_tdata[ 9: 2]
                                    }),
                .s_axi4s_tvalid     (axi4s_rgb_tvalid),
                .s_axi4s_tready     (axi4s_rgb_tready),
                
                .s_wb_rst_i         (wb_peri_rst_i),
                .s_wb_clk_i         (wb_peri_clk_i),
                .s_wb_adr_i         (wb_peri_adr_i[7:0]),
                .s_wb_dat_o         (wb_vdmaw_dat_o),
                .s_wb_dat_i         (wb_peri_dat_i),
                .s_wb_we_i          (wb_peri_we_i),
                .s_wb_sel_i         (wb_peri_sel_i),
                .s_wb_stb_i         (wb_vdmaw_stb_i),
                .s_wb_ack_o         (wb_vdmaw_ack_o)
            );
    
    
    // read は未使用
    assign axi4_mem0_arid     = 0;
    assign axi4_mem0_araddr   = 0;
    assign axi4_mem0_arburst  = 0;
    assign axi4_mem0_arcache  = 0;
    assign axi4_mem0_arlen    = 0;
    assign axi4_mem0_arlock   = 0;
    assign axi4_mem0_arprot   = 0;
    assign axi4_mem0_arqos    = 0;
    assign axi4_mem0_arregion = 0;
    assign axi4_mem0_arsize   = 0;
    assign axi4_mem0_arvalid  = 0;
    assign axi4_mem0_rready   = 0;
    
    
    
    
    
    // -----------------------------
    //  position_calc
    // -----------------------------
    
    wire    [31:0]              position_diff;
    wire                        position_valid;
    
    wire    [WB_DAT_WIDTH-1:0]  wb_posc_dat_o;
    wire                        wb_posc_stb_i;
    wire                        wb_posc_ack_o;
    
    diff_calc
            #(
                .WB_ADR_WIDTH       (8),
                .WB_DAT_WIDTH       (WB_DAT_WIDTH),
                .ASYNC              (0),
                .IN_WIDTH           (IMG_ANGLE_WIDTH),
                .OUT_WIDTH          (32),
                .GAIN_WIDTH         (18),
                .Q_WIDTH            (16),
                .INIT_ENABLE        (1),
                .INIT_TARGET        (0),
                .INIT_GAIN          (18'h10000)
            )
        i_diff_calc
            (
                .s_wb_rst_i         (wb_peri_rst_i),
                .s_wb_clk_i         (wb_peri_clk_i),
                .s_wb_adr_i         (wb_peri_adr_i[7:0]),
                .s_wb_dat_o         (wb_posc_dat_o),
                .s_wb_dat_i         (wb_peri_dat_i),
                .s_wb_we_i          (wb_peri_we_i),
                .s_wb_sel_i         (wb_peri_sel_i),
                .s_wb_stb_i         (wb_posc_stb_i),
                .s_wb_ack_o         (wb_posc_ack_o),
                
                .in_reset           (wb_peri_rst_i),
                .in_clk             (wb_peri_clk_i),
                .in_data            (image_angle),
                .in_valid           (image_valid),
                
                .out_reset          (wb_peri_rst_i),
                .out_clk            (wb_peri_clk_i),
                .out_data           (position_diff),
                .out_valid          (position_valid)
            );
    
    
    // -----------------------------
    //  stepper moter control
    // -----------------------------
    
    wire                            stmc_out_en;
    wire                            stmc_out_a;
    wire                            stmc_out_b;
    
    wire                            stmc_update;
    wire    signed  [47:0]          stmc_cur_x;
    wire    signed  [24:0]          stmc_cur_v;
    wire    signed  [24:0]          stmc_cur_a;
    wire    signed  [47:0]          stmc_target_x;
    wire    signed  [24:0]          stmc_target_v;
    wire    signed  [24:0]          stmc_target_a;
    
    wire    [WB_DAT_WIDTH-1:0]      wb_stmc_dat_o;
    wire                            wb_stmc_stb_i;
    wire                            wb_stmc_ack_o;
    
    stepper_motor_control
            #(
                .WB_ADR_WIDTH       (8),
                .WB_DAT_SIZE        (WB_DAT_SIZE),
                
                .Q_WIDTH            (24),       // 小数点サイズ
                .MICROSTEP_WIDTH    (12),
                .X_WIDTH            (48),
                .V_WIDTH            (24),
                .A_WIDTH            (24),
                .X_DIFF_WIDTH       (32),
                
                .INIT_CTL_ENABLE    (1'b0),
                .INIT_CTL_TARGET    (3'b0),
                .INIT_CTL_PWM       (2'b11),
                .INIT_TARGET_X      (0),
                .INIT_TARGET_V      (0),
                .INIT_TARGET_A      (0),
                .INIT_MAX_V         (1000),
                .INIT_MAX_A         (100),
                .INIT_MAX_A_NEAR    (120)
            )
        i_stepper_motor_control
            (
                .reset              (wb_peri_rst_i),
                .clk                (wb_peri_clk_i),
                
                .s_wb_adr_i         (wb_peri_adr_i[7:0]),
                .s_wb_dat_i         (wb_peri_dat_i),
                .s_wb_dat_o         (wb_stmc_dat_o),
                .s_wb_we_i          (wb_peri_we_i),
                .s_wb_sel_i         (wb_peri_sel_i),
                .s_wb_stb_i         (wb_stmc_stb_i),
                .s_wb_ack_o         (wb_stmc_ack_o),
                
                .in_x_diff          (position_diff),
                .in_valid           (position_valid),
                
                .motor_en           (stmc_out_en),
                .motor_a            (stmc_out_a),
                .motor_b            (stmc_out_b),
                
                .monitor_update     (stmc_update),
                .monitor_cur_x      (stmc_cur_x),
                .monitor_cur_v      (stmc_cur_v),
                .monitor_cur_a      (stmc_cur_a),
                .monitor_target_x   (stmc_target_x),
                .monitor_target_v   (stmc_target_v),
                .monitor_target_a   (stmc_target_a)
            );
    
    assign pmod0[0]   = stmc_out_a;
    assign pmod0[1]   = stmc_out_b;
    assign pmod0[2]   = stmc_out_en;
    assign pmod0[7:3] = 0;
    
    
    
    // ----------------------------------------
    //  logging
    // ----------------------------------------
    
    // image log
    /*
    wire    [IMG_ANGLE_WIDTH-1:0]   log_image_angle;
    wire                            log_image_valid;
    wire                            log_image_ready;
    
    jelly_data_async
            #(
                .ASYNC          (1),
                .DATA_WIDTH     (IMG_ANGLE_WIDTH)
            )
        i_data_async
            (
                .s_reset        (~axi4s_cam_aresetn),
                .s_clk          (axi4s_cam_aclk),
                .s_data         (image_angle),
                .s_valid        (image_valid),
                .s_ready        (),
                
                .m_reset        (wb_peri_rst_i),
                .m_clk          (wb_peri_clk_i),
                .m_data         (log_image_angle),
                .m_valid        (log_image_valid),
                .m_ready        (log_image_ready)
            );
    */
    
    wire    [WB_DAT_WIDTH-1:0]  wb_log0_dat_o;
    wire                        wb_log0_stb_i;
    wire                        wb_log0_ack_o;
    
    jelly_data_logger_fifo
            #(
                .WB_ADR_WIDTH       (8),
                .WB_DAT_WIDTH       (WB_DAT_WIDTH),
                .DATA_WIDTH         (IMG_ANGLE_WIDTH),
                .TIMER_WIDTH        (48),
                .FIFO_ASYNC         (0),
                .FIFO_PTR_WIDTH     (6),
                .FIFO_RAM_TYPE      ("distributed")
            )
        i_data_logger_fifo_img
            (
                .reset              (wb_peri_rst_i),
                .clk                (wb_peri_clk_i),
                
                .s_data             (image_angle),
                .s_valid            (image_valid),
                .s_ready            (),
                
                .s_wb_rst_i         (wb_peri_rst_i),
                .s_wb_clk_i         (wb_peri_clk_i),
                .s_wb_adr_i         (wb_peri_adr_i[7:0]),
                .s_wb_dat_o         (wb_log0_dat_o),
                .s_wb_dat_i         (wb_peri_dat_i),
                .s_wb_we_i          (wb_peri_we_i),
                .s_wb_sel_i         (wb_peri_sel_i),
                .s_wb_stb_i         (wb_log0_stb_i),
                .s_wb_ack_o         (wb_log0_ack_o)
            );
    
    
    // motor log
    
    wire    [8*WB_DAT_WIDTH-1:0]    log_stmc_data;
    wire                            log_stmc_valid = stmc_update;
    
    assign log_stmc_data[0*WB_DAT_WIDTH +: WB_DAT_WIDTH] = (stmc_cur_x >> (0*WB_DAT_WIDTH));
    assign log_stmc_data[1*WB_DAT_WIDTH +: WB_DAT_WIDTH] = (stmc_cur_x >> (1*WB_DAT_WIDTH));
    assign log_stmc_data[2*WB_DAT_WIDTH +: WB_DAT_WIDTH] = stmc_cur_v;
    assign log_stmc_data[3*WB_DAT_WIDTH +: WB_DAT_WIDTH] = stmc_cur_a;
    assign log_stmc_data[4*WB_DAT_WIDTH +: WB_DAT_WIDTH] = (stmc_target_x >> (0*WB_DAT_WIDTH));
    assign log_stmc_data[5*WB_DAT_WIDTH +: WB_DAT_WIDTH] = (stmc_target_x >> (1*WB_DAT_WIDTH));
    assign log_stmc_data[6*WB_DAT_WIDTH +: WB_DAT_WIDTH] = stmc_target_v;
    assign log_stmc_data[7*WB_DAT_WIDTH +: WB_DAT_WIDTH] = stmc_target_a;
    
    
    wire    [WB_DAT_WIDTH-1:0]  wb_log1_dat_o;
    wire                        wb_log1_stb_i;
    wire                        wb_log1_ack_o;
    
    jelly_data_logger_fifo
            #(
                .WB_ADR_WIDTH       (8),
                .WB_DAT_WIDTH       (WB_DAT_WIDTH),
                .DATA_WIDTH         (8*WB_DAT_WIDTH),
                .TIMER_WIDTH        (48),
                .FIFO_ASYNC         (0),
                .FIFO_PTR_WIDTH     (6),
                .FIFO_RAM_TYPE      ("distributed")
            )
        i_data_logger_fifo_motor
            (
                .reset              (wb_peri_rst_i),
                .clk                (wb_peri_clk_i),
                
                .s_data             (log_stmc_data),
                .s_valid            (log_stmc_valid),
                .s_ready            (),
                
                .s_wb_rst_i         (wb_peri_rst_i),
                .s_wb_clk_i         (wb_peri_clk_i),
                .s_wb_adr_i         (wb_peri_adr_i[7:0]),
                .s_wb_dat_o         (wb_log1_dat_o),
                .s_wb_dat_i         (wb_peri_dat_i),
                .s_wb_we_i          (wb_peri_we_i),
                .s_wb_sel_i         (wb_peri_sel_i),
                .s_wb_stb_i         (wb_log1_stb_i),
                .s_wb_ack_o         (wb_log1_ack_o)
            );
    
    
    
    
    
    // ----------------------------------------
    //  WISHBONE address decoder
    // ----------------------------------------
    
    assign wb_gid_stb_i   = wb_peri_stb_i & (wb_peri_adr_i[24:9]  == 16'h000_0);   // 0x80000000-0x80000fff
    assign wb_fmtr_stb_i  = wb_peri_stb_i & (wb_peri_adr_i[24:9]  == 16'h001_0);   // 0x80010000-0x80010fff
    assign wb_prmup_stb_i = wb_peri_stb_i & (wb_peri_adr_i[24:9]  == 16'h001_1);   // 0x80011000-0x80011fff
    assign wb_vdmaw_stb_i = wb_peri_stb_i & (wb_peri_adr_i[24:9]  == 16'h002_1);   // 0x80021000-0x80021fff
    assign wb_imgp_stb_i  = wb_peri_stb_i & (wb_peri_adr_i[24:13] == 12'h003);     // 0x80030000-0x8003ffff
    assign wb_stmc_stb_i  = wb_peri_stb_i & (wb_peri_adr_i[24:9]  == 16'h004_1);   // 0x80041000-0x80041fff
    assign wb_posc_stb_i  = wb_peri_stb_i & (wb_peri_adr_i[24:9]  == 16'h004_2);   // 0x80042000-0x80042fff
    assign wb_log0_stb_i  = wb_peri_stb_i & (wb_peri_adr_i[24:9]  == 16'h007_0);   // 0x80070000-0x80070fff
    assign wb_log1_stb_i  = wb_peri_stb_i & (wb_peri_adr_i[24:9]  == 16'h007_1);   // 0x80071000-0x80071fff
    
    assign wb_peri_dat_o  = wb_gid_stb_i   ? wb_gid_dat_o   :
                            wb_fmtr_stb_i  ? wb_fmtr_dat_o  :
                            wb_prmup_stb_i ? wb_prmup_dat_o :
                            wb_vdmaw_stb_i ? wb_vdmaw_dat_o :
                            wb_imgp_stb_i  ? wb_imgp_dat_o  :
                            wb_stmc_stb_i  ? wb_stmc_dat_o  :
                            wb_posc_stb_i  ? wb_posc_dat_o  :
                            wb_log0_stb_i  ? wb_log0_dat_o  :
                            wb_log1_stb_i  ? wb_log1_dat_o  :
                            {WB_DAT_WIDTH{1'b0}};
    
    assign wb_peri_ack_o  = wb_gid_stb_i   ? wb_gid_ack_o   :
                            wb_fmtr_stb_i  ? wb_fmtr_ack_o  :
                            wb_prmup_stb_i ? wb_prmup_ack_o :
                            wb_vdmaw_stb_i ? wb_vdmaw_ack_o :
                            wb_imgp_stb_i  ? wb_imgp_ack_o  :
                            wb_stmc_stb_i  ? wb_stmc_ack_o  :
                            wb_posc_stb_i  ? wb_posc_ack_o  :
                            wb_log0_stb_i  ? wb_log0_ack_o  :
                            wb_log1_stb_i  ? wb_log1_ack_o  :
                            wb_peri_stb_i;
    
    
    
    // ----------------------------------------
    //  Debug
    // ----------------------------------------
    
    reg     [31:0]      reg_counter_rxbyteclkhs;
    always @(posedge rxbyteclkhs)   reg_counter_rxbyteclkhs <= reg_counter_rxbyteclkhs + 1;
    
    reg     [31:0]      reg_counter_clk200;
    always @(posedge sys_clk200)    reg_counter_clk200 <= reg_counter_clk200 + 1;
    
    reg     [31:0]      reg_counter_clk100;
    always @(posedge sys_clk100)    reg_counter_clk100 <= reg_counter_clk100 + 1;
    
    reg     [31:0]      reg_counter_peri_aclk;
    always @(posedge axi4l_peri_aclk)   reg_counter_peri_aclk <= reg_counter_peri_aclk + 1;
    
    reg     [31:0]      reg_counter_mem_aclk;
    always @(posedge axi4_mem_aclk) reg_counter_mem_aclk <= reg_counter_mem_aclk + 1;
    
    reg     frame_toggle = 0;
    always @(posedge axi4s_cam_aclk) begin
        if ( axi4s_csi2_tuser[0] && axi4s_csi2_tvalid && axi4s_csi2_tready ) begin
            frame_toggle <= ~frame_toggle;
        end
    end
    
    
    assign led[0] = reg_counter_rxbyteclkhs[24];
    assign led[1] = reg_counter_peri_aclk[24]; // reg_counter_clk200[24];
    assign led[2] = reg_counter_mem_aclk[24];  // reg_counter_clk100[24];
    assign led[3] = frame_toggle;
    
    assign pmod1[0]   = frame_toggle;
    assign pmod1[1]   = reg_counter_rxbyteclkhs[5];
    assign pmod1[2]   = reg_counter_clk200[5];
    assign pmod1[3]   = reg_counter_clk100[5];
    assign pmod1[7:4] = 0;
    
    
endmodule


`default_nettype wire

