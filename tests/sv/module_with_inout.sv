`timescale 1ns/1ps

module module_with_inout (
    input  wire a,
    output wire b,
    inout  wire pad
);

    // Simple combinational connection
    assign b = a;

    // Example bidirectional pad (tri-stated by default)
    assign pad = 1'bz;

endmodule

