`timescale 1ns/1ps

`define DATA_WIDTH 8
`define DATA_WIDTH_SV_H 8'hac
`define DATA_WIDTH_SV_D 8'd512
`define DATA_WIDTH_SV_B 8'b00101000

module basic_module
(
    input  logic                      clk,
    input  logic                      rst_n,
    input  logic [`DATA_WIDTH-1:0]    data_in,
    output logic [`DATA_WIDTH- 1:0]    data_out,
    input [`DATA_WIDTH_SV_H-1:0]  data_sv_h,
    input [`DATA_WIDTH_SV_D -1:0]  data_sv_d,
    input [`DATA_WIDTH_SV_B - 1:0]  data_sv_b,
    input [(1  <<12 ) - 1:0] data_sv_shift,
    input [( 256 * 4 ) / 8-1:0] data_sv_mul_div
);

    // Simple combinational pass-through
    assign data_out = data_in;

endmodule

