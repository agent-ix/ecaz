.section .text.ecaz::quant::prod::ProdQuantizer::score_ip_codes_lite,"ax",@progbits
	.globl	ecaz::quant::prod::ProdQuantizer::score_ip_codes_lite
	.p2align	4
.type	ecaz::quant::prod::ProdQuantizer::score_ip_codes_lite,@function
ecaz::quant::prod::ProdQuantizer::score_ip_codes_lite:
	.cfi_startproc
	sub sp, sp, #80
	.cfi_def_cfa_offset 80
	stp x29, x30, [sp, #64]
	add x29, sp, #64
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_remember_state
	ldr x10, [x0, #80]
	ldrb w11, [x0, #96]
	mov x8, x2
	cmp w11, #4
	b.ne .LBB4283_4
	cmp x10, #1536
	b.ne .LBB4283_5
	mov w12, #768
	stp x8, x12, [sp]
	cmp x8, #768
	b.ne .LBB4283_15
	mov x9, xzr
	mov w2, #768
	b .LBB4283_13
.LBB4283_4:
	cmp w11, #0
	cset w9, ne
	sub w9, w11, w9
	b .LBB4283_6
.LBB4283_5:
	mov w9, #3
.LBB4283_6:
	mul x9, x9, x10
	lsr x12, x9, #3
	tst x9, #0x7
	lsr x9, x10, #3
	cinc x2, x12, ne
	tst x10, #0x7
	cinc x9, x9, ne
	add x12, x9, x2
	stp x8, x12, [sp]
	cmp x8, x12
	b.ne .LBB4283_15
	cmp w11, #4
	b.ne .LBB4283_10
	cmp x10, #1536
	b.ne .LBB4283_11
	mov w8, #768
	mov x9, xzr
	b .LBB4283_13
.LBB4283_10:
	cmp w11, #0
	cset w8, ne
	sub w8, w11, w8
	b .LBB4283_12
.LBB4283_11:
	mov w8, #3
.LBB4283_12:
	mul x8, x8, x10
	lsr x10, x8, #3
	tst x8, #0x7
	cinc x8, x10, ne
.LBB4283_13:
	add x9, x8, x9
	stp x4, x9, [sp]
	cmp x4, x9
	b.ne .LBB4283_16
	mov x4, x8
	.cfi_def_cfa wsp, 80
	ldp x29, x30, [sp, #64]
	add sp, sp, #80
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	b ecaz::quant::prod::ProdQuantizer::score_ip_mse_codes
.LBB4283_15:
	.cfi_restore_state
	stp x8, x12, [sp, #16]
	b .LBB4283_17
.LBB4283_16:
	stp x4, x9, [sp, #16]
.LBB4283_17:
	adrp x9, :got:<usize as core::fmt::Display>::fmt
	add x8, sp, #16
	adrp x3, .Lanon.8f2aa15b57af37b749e6b281cbc37a85.11568
	add x3, x3, :lo12:.Lanon.8f2aa15b57af37b749e6b281cbc37a85.11568
	adrp x5, .Lanon.8f2aa15b57af37b749e6b281cbc37a85.11569
	add x5, x5, :lo12:.Lanon.8f2aa15b57af37b749e6b281cbc37a85.11569
	ldr x9, [x9, :got_lo12:<usize as core::fmt::Display>::fmt]
	mov x1, sp
	add x2, sp, #8
	add x4, sp, #32
	mov w0, wzr
	stp x8, x9, [sp, #32]
	add x8, sp, #24
	stp x8, x9, [sp, #48]
	bl core::panicking::assert_failed::<usize, usize>
