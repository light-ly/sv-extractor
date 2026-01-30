`timescale 1ns/1ps

`define DATA_WIDTH 8

module basic_module
(
    input  logic                      clk,
    input  logic                      rst_n,
    input  logic [`DATA_WIDTH-1:0]    data_in,
    output logic [`DATA_WIDTH-1:0]    data_out
);

    // Simple combinational pass-through
    assign data_out = data_in;

endmodule

