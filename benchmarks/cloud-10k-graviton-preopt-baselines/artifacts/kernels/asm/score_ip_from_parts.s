.section .text.ecaz::quant::prod::ProdQuantizer::score_ip_from_parts,"ax",@progbits
	.globl	ecaz::quant::prod::ProdQuantizer::score_ip_from_parts
	.p2align	4
.type	ecaz::quant::prod::ProdQuantizer::score_ip_from_parts,@function
ecaz::quant::prod::ProdQuantizer::score_ip_from_parts:
	.cfi_startproc
	sub sp, sp, #80
	.cfi_def_cfa_offset 80
	stp x29, x30, [sp, #64]
	add x29, sp, #64
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_remember_state
	ldr x9, [x0, #80]
	ldrb w8, [x0, #96]
	cmp w8, #4
	b.ne .LBB4284_3
	cmp x9, #1536
	b.ne .LBB4284_4
	mov w8, #768
	mov x5, xzr
	b .LBB4284_6
.LBB4284_3:
	cmp w8, #0
	cset w10, ne
	sub w8, w8, w10
	b .LBB4284_5
.LBB4284_4:
	mov w8, #3
.LBB4284_5:
	mul x8, x8, x9
	lsr x10, x8, #3
	tst x8, #0x7
	cinc x8, x10, ne
	lsr x10, x9, #3
	tst x9, #0x7
	cinc x5, x10, ne
.LBB4284_6:
	add x9, x8, x5
	stp x3, x9, [sp]
	cmp x3, x9
	b.ne .LBB4284_8
	add x4, x2, x8
	mov x3, x8
	.cfi_def_cfa wsp, 80
	ldp x29, x30, [sp, #64]
	add sp, sp, #80
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	b ecaz::quant::prod::ProdQuantizer::score_ip_from_split_parts
.LBB4284_8:
	.cfi_restore_state
	stp x3, x9, [sp, #16]
	adrp x9, :got:<usize as core::fmt::Display>::fmt
	add x8, sp, #16
	adrp x3, .Lanon.8f2aa15b57af37b749e6b281cbc37a85.11568
	add x3, x3, :lo12:.Lanon.8f2aa15b57af37b749e6b281cbc37a85.11568
	ldr x9, [x9, :got_lo12:<usize as core::fmt::Display>::fmt]
	adrp x5, .Lanon.8f2aa15b57af37b749e6b281cbc37a85.11569
	add x5, x5, :lo12:.Lanon.8f2aa15b57af37b749e6b281cbc37a85.11569
	mov x1, sp
	add x2, sp, #8
	add x4, sp, #32
	mov w0, wzr
	stp x8, x9, [sp, #32]
	add x8, sp, #24
	stp x8, x9, [sp, #48]
	bl core::panicking::assert_failed::<usize, usize>
