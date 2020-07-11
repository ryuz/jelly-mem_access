// ---------------------------------------------------------------------------
//  Jelly  -- The FPGA processing system
//
//
//                                 Copyright (C) 2008-2020 by Ryuji Fuchikami
//                                 https://github.com/ryuz/
// ---------------------------------------------------------------------------



`timescale 1ns / 1ps
`default_nettype none


module video_raw_to_rgb
        #(
            parameter   WB_ADR_WIDTH  = 8,
            parameter   WB_DAT_WIDTH  = 32,
            parameter   WB_SEL_WIDTH  = (WB_DAT_WIDTH / 8),
            
            parameter   DATA_WIDTH    = 10,
            
            parameter   IMG_Y_NUM     = 480,
            parameter   IMG_Y_WIDTH   = 12,
            
            parameter   TUSER_WIDTH   = 1,
            parameter   S_TDATA_WIDTH = DATA_WIDTH,
            parameter   M_TDATA_WIDTH = 4*DATA_WIDTH
        )
        (
            input   wire                        aresetn,
            input   wire                        aclk,
            
            input   wire                        in_update_req,
            
            input   wire                        s_wb_rst_i,
            input   wire                        s_wb_clk_i,
            input   wire    [WB_ADR_WIDTH-1:0]  s_wb_adr_i,
            input   wire    [WB_DAT_WIDTH-1:0]  s_wb_dat_i,
            output  wire    [WB_DAT_WIDTH-1:0]  s_wb_dat_o,
            input   wire                        s_wb_we_i,
            input   wire    [WB_SEL_WIDTH-1:0]  s_wb_sel_i,
            input   wire                        s_wb_stb_i,
            output  wire                        s_wb_ack_o,
            
            input   wire    [TUSER_WIDTH-1:0]   s_axi4s_tuser,
            input   wire                        s_axi4s_tlast,
            input   wire    [S_TDATA_WIDTH-1:0] s_axi4s_tdata,
            input   wire                        s_axi4s_tvalid,
            output  wire                        s_axi4s_tready,
            
            output  wire    [TUSER_WIDTH-1:0]   m_axi4s_tuser,
            output  wire                        m_axi4s_tlast,
            output  wire    [M_TDATA_WIDTH-1:0] m_axi4s_tdata,
            output  wire                        m_axi4s_tvalid,
            input   wire                        m_axi4s_tready
        );
    
    localparam  USE_VALID = 0;
    
    
    wire                                reset = ~aresetn;
    wire                                clk   = aclk;
    wire                                cke;
    
    wire                                img_src_line_first;
    wire                                img_src_line_last;
    wire                                img_src_pixel_first;
    wire                                img_src_pixel_last;
    wire                                img_src_de;
    wire    [TUSER_WIDTH-1:0]           img_src_user;
    wire    [S_TDATA_WIDTH-1:0]         img_src_data;
    wire                                img_src_valid;
    
    wire                                img_sink_line_first;
    wire                                img_sink_line_last;
    wire                                img_sink_pixel_first;
    wire                                img_sink_pixel_last;
    wire                                img_sink_de;
    wire    [TUSER_WIDTH-1:0]           img_sink_user;
    wire    [M_TDATA_WIDTH-1:0]         img_sink_data;
    wire                                img_sink_valid;
    
    // img
    jelly_axi4s_img
            #(
                .TUSER_WIDTH            (TUSER_WIDTH),
                .S_TDATA_WIDTH          (DATA_WIDTH),
                .M_TDATA_WIDTH          (4*DATA_WIDTH),
                .IMG_Y_NUM              (IMG_Y_NUM),
                .IMG_Y_WIDTH            (IMG_Y_WIDTH),
                .BLANK_Y_WIDTH          (8),
                .IMG_CKE_BUFG           (0)
            )
        jelly_axi4s_img
            (
                .reset                  (reset),
                .clk                    (clk),
                
                .param_blank_num        (8'h00),
                
                .s_axi4s_tdata          (s_axi4s_tdata),
                .s_axi4s_tlast          (s_axi4s_tlast),
                .s_axi4s_tuser          (s_axi4s_tuser),
                .s_axi4s_tvalid         (s_axi4s_tvalid),
                .s_axi4s_tready         (s_axi4s_tready),
                
                .m_axi4s_tdata          (m_axi4s_tdata),
                .m_axi4s_tlast          (m_axi4s_tlast),
                .m_axi4s_tuser          (m_axi4s_tuser),
                .m_axi4s_tvalid         (m_axi4s_tvalid),
                .m_axi4s_tready         (m_axi4s_tready),
                
                
                .img_cke                (cke),
                
                .src_img_line_first     (img_src_line_first),
                .src_img_line_last      (img_src_line_last),
                .src_img_pixel_first    (img_src_pixel_first),
                .src_img_pixel_last     (img_src_pixel_last),
                .src_img_de             (img_src_de),
                .src_img_user           (img_src_user),
                .src_img_data           (img_src_data),
                .src_img_valid          (img_src_valid),
                
                .sink_img_line_first    (img_sink_line_first),
                .sink_img_line_last     (img_sink_line_last),
                .sink_img_pixel_first   (img_sink_pixel_first),
                .sink_img_pixel_last    (img_sink_pixel_last),
                .sink_img_user          (img_sink_user),
                .sink_img_de            (img_sink_de),
                .sink_img_data          (img_sink_data),
                .sink_img_valid         (img_sink_valid)
            );
    
    
    
    // demosaic
    wire                                img_demos_line_first;
    wire                                img_demos_line_last;
    wire                                img_demos_pixel_first;
    wire                                img_demos_pixel_last;
    wire                                img_demos_de;
    wire    [TUSER_WIDTH-1:0]           img_demos_user;
    wire    [DATA_WIDTH-1:0]            img_demos_raw;
    wire    [DATA_WIDTH-1:0]            img_demos_r;
    wire    [DATA_WIDTH-1:0]            img_demos_g;
    wire    [DATA_WIDTH-1:0]            img_demos_b;
    wire                                img_demos_valid;
    
    wire    [WB_DAT_WIDTH-1:0]          wb_demos_dat_o;
    wire                                wb_demos_stb_i;
    wire                                wb_demos_ack_o;
    
    jelly_img_demosaic_acpi
            #(
                .USER_WIDTH             (TUSER_WIDTH),
                .DATA_WIDTH             (DATA_WIDTH),
                .MAX_X_NUM              (4096),
                .RAM_TYPE               ("block"),
                .USE_VALID              (USE_VALID),
                
                .WB_ADR_WIDTH           (6),
                .WB_DAT_WIDTH           (WB_DAT_WIDTH),
                
                .INIT_PARAM_PHASE       (2'b11)
            )
        i_img_demosaic_acpi
            (
                .reset                  (reset),
                .clk                    (clk),
                .cke                    (cke),
                
                .in_update_req          (in_update_req),
                
                .s_wb_rst_i             (s_wb_rst_i),
                .s_wb_clk_i             (s_wb_clk_i),
                .s_wb_adr_i             (s_wb_adr_i[5:0]),
                .s_wb_dat_i             (s_wb_dat_i),
                .s_wb_dat_o             (wb_demos_dat_o),
                .s_wb_we_i              (s_wb_we_i),
                .s_wb_sel_i             (s_wb_sel_i),
                .s_wb_stb_i             (wb_demos_stb_i),
                .s_wb_ack_o             (wb_demos_ack_o),
                
                .s_img_line_first       (img_src_line_first),
                .s_img_line_last        (img_src_line_last),
                .s_img_pixel_first      (img_src_pixel_first),
                .s_img_pixel_last       (img_src_pixel_last),
                .s_img_de               (img_src_de),
                .s_img_user             (img_src_user),
                .s_img_raw              (img_src_data),
                .s_img_valid            (img_src_valid),
                
                .m_img_line_first       (img_demos_line_first),
                .m_img_line_last        (img_demos_line_last),
                .m_img_pixel_first      (img_demos_pixel_first),
                .m_img_pixel_last       (img_demos_pixel_last),
                .m_img_de               (img_demos_de),
                .m_img_user             (img_demos_user),
                .m_img_raw              (img_demos_raw),
                .m_img_r                (img_demos_r),
                .m_img_g                (img_demos_g),
                .m_img_b                (img_demos_b),
                .m_img_valid            (img_demos_valid)
            );
    
    
    // color matrix
    wire    [WB_DAT_WIDTH-1:0]          wb_colmat_dat_o;
    wire                                wb_colmat_stb_i;
    wire                                wb_colmat_ack_o;
    
    jelly_img_color_matrix
            #(
                .USER_WIDTH             (DATA_WIDTH + TUSER_WIDTH),
                .DATA_WIDTH             (DATA_WIDTH),
                .INTERNAL_WIDTH         (DATA_WIDTH+2),
                
                .COEFF_INT_WIDTH        (9),
                .COEFF_FRAC_WIDTH       (16),
                .COEFF3_INT_WIDTH       (9),
                .COEFF3_FRAC_WIDTH      (16),
                .STATIC_COEFF           (1),
                .DEVICE                 ("7SERIES"),
                
                .WB_ADR_WIDTH           (6),
                .WB_DAT_WIDTH           (WB_DAT_WIDTH),
                
                .INIT_PARAM_MATRIX00    (2 << 16),
                .INIT_PARAM_MATRIX01    (0),
                .INIT_PARAM_MATRIX02    (0),
                .INIT_PARAM_MATRIX03    (0),
                .INIT_PARAM_MATRIX10    (0),
                .INIT_PARAM_MATRIX11    (1 << 16),
                .INIT_PARAM_MATRIX12    (0),
                .INIT_PARAM_MATRIX13    (0),
                .INIT_PARAM_MATRIX20    (0),
                .INIT_PARAM_MATRIX21    (0),
                .INIT_PARAM_MATRIX22    (2 << 16),
                .INIT_PARAM_MATRIX23    (0),
                .INIT_PARAM_CLIP_MIN0   ({DATA_WIDTH{1'b0}}),
                .INIT_PARAM_CLIP_MAX0   ({DATA_WIDTH{1'b1}}),
                .INIT_PARAM_CLIP_MIN1   ({DATA_WIDTH{1'b0}}),
                .INIT_PARAM_CLIP_MAX1   ({DATA_WIDTH{1'b1}}),
                .INIT_PARAM_CLIP_MIN2   ({DATA_WIDTH{1'b0}}),
                .INIT_PARAM_CLIP_MAX2   ({DATA_WIDTH{1'b1}})
            )
        i_img_color_matrix
            (
                .reset                  (reset),
                .clk                    (clk),
                .cke                    (cke),
                
                .in_update_req          (in_update_req),
                
                .s_wb_rst_i             (s_wb_rst_i),
                .s_wb_clk_i             (s_wb_clk_i),
                .s_wb_adr_i             (s_wb_adr_i[5:0]),
                .s_wb_dat_i             (s_wb_dat_i),
                .s_wb_dat_o             (wb_colmat_dat_o),
                .s_wb_we_i              (s_wb_we_i),
                .s_wb_sel_i             (s_wb_sel_i),
                .s_wb_stb_i             (wb_colmat_stb_i),
                .s_wb_ack_o             (wb_colmat_ack_o),
                
                .s_img_line_first       (img_demos_line_first),
                .s_img_line_last        (img_demos_line_last),
                .s_img_pixel_first      (img_demos_pixel_first),
                .s_img_pixel_last       (img_demos_pixel_last),
                .s_img_de               (img_demos_de),
                .s_img_user             ({img_demos_user, img_demos_raw}),
                .s_img_color0           (img_demos_r),
                .s_img_color1           (img_demos_g),
                .s_img_color2           (img_demos_b),
                .s_img_valid            (img_demos_valid),
                
                .m_img_line_first       (img_sink_line_first),
                .m_img_line_last        (img_sink_line_last),
                .m_img_pixel_first      (img_sink_pixel_first),
                .m_img_pixel_last       (img_sink_pixel_last),
                .m_img_de               (img_sink_de),
                .m_img_user             ({img_sink_user, img_sink_data[DATA_WIDTH*3 +: DATA_WIDTH]}),
                .m_img_color0           (img_sink_data[DATA_WIDTH*2 +: DATA_WIDTH]),
                .m_img_color1           (img_sink_data[DATA_WIDTH*1 +: DATA_WIDTH]),
                .m_img_color2           (img_sink_data[DATA_WIDTH*0 +: DATA_WIDTH]),
                .m_img_valid            (img_sink_valid)
            );
    
    assign wb_demos_stb_i  = s_wb_stb_i & (s_wb_adr_i[WB_ADR_WIDTH-1:6] == 0);
    assign wb_colmat_stb_i = s_wb_stb_i & (s_wb_adr_i[WB_ADR_WIDTH-1:6] == 1);
    
    assign s_wb_dat_o      = wb_demos_stb_i  ? wb_demos_dat_o  :
                             wb_colmat_stb_i ? wb_colmat_dat_o :
                             32'h0000_0000;
    
    assign s_wb_ack_o      = wb_demos_stb_i  ? wb_demos_ack_o  :
                             wb_colmat_stb_i ? wb_colmat_ack_o :
                             s_wb_stb_i;
    
    
    
endmodule



`default_nettype wire



// end of file
