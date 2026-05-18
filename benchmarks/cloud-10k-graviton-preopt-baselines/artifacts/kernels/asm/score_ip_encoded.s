.section .text.ecaz::quant::prod::ProdQuantizer::score_ip_encoded,"ax",@progbits
	.globl	ecaz::quant::prod::ProdQuantizer::score_ip_encoded
	.p2align	4
.type	ecaz::quant::prod::ProdQuantizer::score_ip_encoded,@function
ecaz::quant::prod::ProdQuantizer::score_ip_encoded:
	.cfi_startproc
	sub sp, sp, #80
	.cfi_def_cfa_offset 80
	stp x29, x30, [sp, #64]
	add x29, sp, #64
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_remember_state
	cmp x3, #3
	b.ls .LBB4280_9
	ldr x10, [x0, #80]
	ldrb w9, [x0, #96]
	mov x8, x2
	cmp w9, #4
	b.ne .LBB4280_4
	cmp x10, #1536
	b.ne .LBB4280_5
	mov w9, #768
	mov x5, xzr
	b .LBB4280_7
.LBB4280_4:
	cmp w9, #0
	cset w11, ne
	sub w9, w9, w11
	b .LBB4280_6
.LBB4280_5:
	mov w9, #3
.LBB4280_6:
	mul x9, x9, x10
	lsr x11, x9, #3
	tst x9, #0x7
	cinc x9, x11, ne
	lsr x11, x10, #3
	tst x10, #0x7
	cinc x5, x11, ne
.LBB4280_7:
	ldr s0, [x8]
	add x10, x9, #4
	add x11, x10, x5
	stp x3, x11, [sp]
	cmp x3, x11
	b.ne .LBB4280_10
	add x2, x8, #4
	add x4, x8, x10
	mov x3, x9
	.cfi_def_cfa wsp, 80
	ldp x29, x30, [sp, #64]
	add sp, sp, #80
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	b ecaz::quant::prod::ProdQuantizer::score_ip_from_split_parts
.LBB4280_9:
	.cfi_restore_state
	str x3, [sp, #24]
	adrp x9, :got:<usize as core::fmt::Display>::fmt
	add x8, sp, #24
	adrp x0, .Lanon.8f2aa15b57af37b749e6b281cbc37a85.11561
	add x0, x0, :lo12:.Lanon.8f2aa15b57af37b749e6b281cbc37a85.11561
	ldr x9, [x9, :got_lo12:<usize as core::fmt::Display>::fmt]
	adrp x2, .Lanon.8f2aa15b57af37b749e6b281cbc37a85.11563
	add x2, x2, :lo12:.Lanon.8f2aa15b57af37b749e6b281cbc37a85.11563
	add x1, sp, #32
	stp x8, x9, [sp, #32]
	bl core::panicking::panic_fmt
.LBB4280_10:
	stp x3, x11, [sp, #16]
	adrp x9, :got:<usize as core::fmt::Display>::fmt
	add x8, sp, #16
	adrp x3, .Lanon.8f2aa15b57af37b749e6b281cbc37a85.11562
	add x3, x3, :lo12:.Lanon.8f2aa15b57af37b749e6b281cbc37a85.11562
	ldr x9, [x9, :got_lo12:<usize as core::fmt::Display>::fmt]
	adrp x5, .Lanon.8f2aa15b57af37b749e6b281cbc37a85.11564
	add x5, x5, :lo12:.Lanon.8f2aa15b57af37b749e6b281cbc37a85.11564
	mov x1, sp
	add x2, sp, #8
	add x4, sp, #32
	mov w0, wzr
	stp x8, x9, [sp, #32]
	add x8, sp, #24
	stp x8, x9, [sp, #48]
	bl core::panicking::assert_failed::<usize, usize>
