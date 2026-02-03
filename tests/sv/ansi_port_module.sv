`timescale 1ns/1ps

`define WIDTH 10

module ansi_port_module (
    input  logic                 clk,
    input  logic                 rst_n,
    input  logic [`WIDTH-1:0]     a,
    input  logic [`WIDTH-1:0]     b,
    output logic [`WIDTH-1:0]     sum
);

    // Simple registered adder with active-low reset
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n)
            sum <= '0;
        else
            sum <= a + b;
    end

endmodule

