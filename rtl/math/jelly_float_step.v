// ---------------------------------------------------------------------------
//  Jelly  -- the soft-core processor system
//   浮動小数点の順次インクリメント/デクリメント値生成コア
//
//                                 Copyright (C) 2008-2010 by Ryuji Fuchikami
//                                 http://homepage3.nifty.com/ryuz/
// ---------------------------------------------------------------------------



`timescale 1ns / 1ps
`default_nettype none


// 浮動小数点の順次インクリメント/デクリメント値生成コア
module jelly_float_step
		#(
			parameter	EXP_WIDTH  = 8,
			parameter	FRAC_WIDTH = 23,
			parameter	DATA_WIDTH = 1 + EXP_WIDTH + FRAC_WIDTH
		)
		(
			input	wire						clk,
			input	wire						cke,
			
			input	wire	[DATA_WIDTH-1:0]	param_init,
			input	wire	[DATA_WIDTH-1:0]	param_step,
			
			input	wire						set_param,
			input	wire						increment,
			input	wire						in_valid,
			
			output	wire	[DATA_WIDTH-1:0]	out_data,
			output	wire						out_valid
		);
	
	
	wire								st0_init_sign = param_init[DATA_WIDTH-1];
	wire			[EXP_WIDTH-1:0]		st0_init_exp  = param_init[FRAC_WIDTH +: EXP_WIDTH];
	wire			[FRAC_WIDTH-1:0]	st0_init_frac = param_init[FRAC_WIDTH-1:0];
	wire								st0_step_sign = param_step[DATA_WIDTH-1];
	wire			[EXP_WIDTH-1:0]		st0_step_exp  = param_step[FRAC_WIDTH +: EXP_WIDTH];
	wire			[FRAC_WIDTH-1:0]	st0_step_frac = param_step[FRAC_WIDTH-1:0];
	wire								st0_set_param = set_param;
	wire								st0_increment = increment;
	wire								st0_valid     = in_valid;
	
	reg									st1_init_sign;
	reg				[EXP_WIDTH-1:0]		st1_init_shift;
	reg				[EXP_WIDTH-1:0]		st1_init_exp;
	reg				[FRAC_WIDTH-1:0]	st1_init_frac;
	reg									st1_step_sign;
	reg				[EXP_WIDTH-1:0]		st1_step_shift;
	reg				[FRAC_WIDTH-1:0]	st1_step_frac;
	reg									st1_set_param;
	reg									st1_increment;
	reg									st1_valid;
	
	reg									st2_init_sign;
	reg				[EXP_WIDTH-1:0]		st2_init_exp;
	reg				[FRAC_WIDTH-1:0]	st2_init_frac;
	reg									st2_step_sign;
	reg				[FRAC_WIDTH-1:0]	st2_step_frac;
	reg									st2_set_param;
	reg									st2_increment;
	reg									st2_valid;

	reg									st3_base_sign;
	reg				[EXP_WIDTH-1:0]		st3_base_exp;
	reg		signed	[FRAC_WIDTH+1:0]	st3_base_frac;
	reg		signed	[FRAC_WIDTH+1:0]	st3_step_frac;
	wire	signed	[FRAC_WIDTH+1:0]	st3_inc_frac = st3_base_frac + st3_step_frac;
	reg									st3_valid;

	reg									st4_out_sign;
	reg				[EXP_WIDTH-1:0]		st4_out_exp;
	reg				[FRAC_WIDTH-1:0]	st4_out_frac;
	reg									st4_valid;
		
	always @(posedge clk) begin
		if ( cke ) begin
			// stage 1 (指数部をどちらに合わせるか判別)
			st1_init_sign <= st0_init_sign;
			st1_init_frac <= st0_init_frac;
			st1_step_sign <= st0_step_sign;
			st1_step_frac <= st0_step_frac;
			if ( st0_init_exp >= st0_step_exp ) begin
				st1_init_shift <= st0_init_exp - st0_step_exp;
				st1_init_exp   <= st0_init_exp;
				st1_step_shift <= 0;
			end
			else begin
				st1_init_exp   <= st0_step_exp;
				st1_init_shift <= 0;
				st1_step_shift <= st0_step_exp - st0_init_exp;
			end
			st1_set_param <= st0_set_param;
			st1_increment <= st0_increment;
			st1_valid     <= st0_valid;
			
			
			// stage 2 (桁あわせ)
			st2_init_sign <= st1_init_sign;
			st2_init_exp  <= st1_init_exp;
			st2_init_frac <= ({1'b1, st1_init_frac} >> st1_init_shift);
			st2_step_sign <= st1_step_sign;
			st2_step_frac <= ({1'b1, st1_step_frac} >> st1_step_shift);
			st2_set_param <= st1_set_param;
			st2_increment <= st1_increment & st1_valid;
			st2_valid     <= st1_valid;
			
			
			// stage 3 (インクリメント計算)
			if ( st2_set_param ) begin
				// 初期化
				st3_base_sign <= st2_init_sign;
				st3_base_exp  <= st2_init_exp;
				st3_base_frac <= {2'b01, st2_init_frac};
				st3_step_frac <= (st2_init_sign == st2_step_sign) ? {2'b01, st2_step_frac} : -{2'b01, st2_step_frac};
			end
			else if ( st2_increment ) begin
				// インクリメント
				if ( st3_inc_frac[FRAC_WIDTH+1] == st3_inc_frac[FRAC_WIDTH] ) begin
					// 桁上がり無し
					st3_base_exp  <= st2_init_exp;
					st3_base_frac <= st3_inc_frac;
				end
				else begin
					// 桁上がりあり
					st3_base_exp  <= st2_init_exp + 1'b1;
					st3_base_frac <= (st3_inc_frac >>> 1);
				end
			end
			st3_valid <= st2_valid;
			
			// stage4 (符号整形)
			st4_out_sign <= st3_base_frac[FRAC_WIDTH+1] ? ~st3_base_sign : st3_base_sign;
			st4_out_exp  <= st3_base_exp;
			st4_out_frac <= st3_base_frac[FRAC_WIDTH+1] ? -st3_base_frac : st3_base_frac;
			st4_valid    <= st3_valid;
		end
	end
	
	assign out_data  = {st4_out_sign, st4_out_exp, st4_out_frac[FRAC_WIDTH-1:0]};
	assign out_valid = st4_valid;
	
endmodule


`default_nettype wire


// end of file
