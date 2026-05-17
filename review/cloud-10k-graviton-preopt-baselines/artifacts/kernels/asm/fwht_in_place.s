.section .text.ecaz::quant::hadamard::fwht_in_place,"ax",@progbits
	.globl	ecaz::quant::hadamard::fwht_in_place
	.p2align	4
.type	ecaz::quant::hadamard::fwht_in_place,@function
ecaz::quant::hadamard::fwht_in_place:
	.cfi_startproc
	sub sp, sp, #128
	.cfi_def_cfa_offset 128
	stp x29, x30, [sp, #32]
	stp x28, x27, [sp, #48]
	stp x26, x25, [sp, #64]
	stp x24, x23, [sp, #80]
	stp x22, x21, [sp, #96]
	stp x20, x19, [sp, #112]
	add x29, sp, #32
	.cfi_def_cfa w29, 96
	.cfi_offset w19, -8
	.cfi_offset w20, -16
	.cfi_offset w21, -24
	.cfi_offset w22, -32
	.cfi_offset w23, -40
	.cfi_offset w24, -48
	.cfi_offset w25, -56
	.cfi_offset w26, -64
	.cfi_offset w27, -72
	.cfi_offset w28, -80
	.cfi_offset w30, -88
	.cfi_offset w29, -96
	sub x8, x1, #1
	eor x9, x1, x8
	cmp x9, x8
	b.ls .LBB4332_39
	adrp x8, ecaz::quant::simd::backend::BACKEND
	add x8, x8, :lo12:ecaz::quant::simd::backend::BACKEND
	ldapr w8, [x8]
	cbnz w8, .LBB4332_40
	adrp x8, ecaz::quant::simd::backend::BACKEND+4
	ldrb w8, [x8, :lo12:ecaz::quant::simd::backend::BACKEND+4]
	tbz w8, #0, .LBB4332_41
.LBB4332_3:
	cmp x1, #2
	b.lo .LBB4332_56
	add x8, x0, #16
	mov w9, #1
	b .LBB4332_6
	.p2align	5, , 16
.LBB4332_5:
	mov x9, x10
	cmp x10, x1
	b.hs .LBB4332_56
.LBB4332_6:
	neg x11, x9, lsl #1
	lsl x10, x9, #1
	and x11, x1, x11
	cmp x10, x11
	b.hi .LBB4332_5
	lsl x12, x9, #3
	cmp x9, #3
	b.hi .LBB4332_14
	add x13, x0, x9, lsl #2
	mov x14, x0
	b .LBB4332_11
	.p2align	5, , 16
.LBB4332_9:
	add x15, x14, x9, lsl #2
	ldr d0, [x14]
	ldr d1, [x15]
	fadd v2.2s, v0.2s, v1.2s
	fsub v0.2s, v0.2s, v1.2s
	str d2, [x14]
	str d0, [x15]
.LBB4332_10:
	add x14, x14, x12
	sub x11, x11, x10
	add x13, x13, x12
	cmp x10, x11
	b.hi .LBB4332_5
.LBB4332_11:
	cmp x9, #1
	b.hi .LBB4332_9
	mov x15, xzr
	.p2align	5, , 16
.LBB4332_13:
	ldr s0, [x14, x15, lsl #2]
	ldr s1, [x13, x15, lsl #2]
	fadd s2, s0, s1
	fsub s0, s0, s1
	str s2, [x14, x15, lsl #2]
	str s0, [x13, x15, lsl #2]
	add x15, x15, #1
	cmp x9, x15
	b.ne .LBB4332_13
	b .LBB4332_10
	.p2align	5, , 16
.LBB4332_14:
	sub x15, x9, #4
	and x21, x9, #0x7ffffffffffffffc
	mov x13, xzr
	add x2, x0, x9, lsl #2
	add x19, x8, x9, lsl #2
	mov x23, x0
	and x14, x15, #0xfffffffffffffffc
	lsl x17, x15, #2
	lsr x15, x15, #2
	add x24, x0, #16
	add x14, x14, #5
	and x17, x17, #0xfffffffffffffff0
	cmp x9, x14
	csel x14, x9, x14, hi
	lsl x16, x14, #2
	sub x18, x16, x17
	lsl x16, x9, #2
	add x14, x16, x14, lsl #2
	add x16, x15, #1
	add x18, x0, x18
	sub x3, x14, x17
	orr x14, x21, #0x1
	lsl x20, x16, #2
	and x17, x16, #0x7ffffffffffffffe
	cmp x9, x14
	add x3, x0, x3
	orr x20, x20, #0x4
	csinc x22, x9, x21, hi
	sub x14, x22, x21
	and x6, x22, #0x1
	and x22, x22, #0xfffffffffffffffe
	and x4, x14, #0x6
	and x5, x14, #0xfffffffffffffff8
	sub x7, x14, x6
	sub x21, x22, x21
	add x22, x0, x9, lsl #2
	b .LBB4332_16
	.p2align	5, , 16
.LBB4332_15:
	add x23, x23, x12
	sub x11, x11, x10
	add x13, x13, #1
	add x19, x19, x12
	add x24, x24, x12
	add x22, x22, x12
	cmp x10, x11
	b.hi .LBB4332_5
.LBB4332_16:
	mov x26, xzr
	cbz x15, .LBB4332_21
	mov x25, x24
	mov x27, x19
	.p2align	5, , 16
.LBB4332_18:
	ldur q0, [x25, #-16]
	ldur q1, [x27, #-16]
	add x26, x26, #2
	fadd v2.4s, v0.4s, v1.4s
	fsub v0.4s, v0.4s, v1.4s
	stur q2, [x25, #-16]
	stur q0, [x27, #-16]
	ldr q0, [x25]
	ldr q1, [x27]
	fadd v2.4s, v0.4s, v1.4s
	fsub v0.4s, v0.4s, v1.4s
	str q2, [x25], #32
	str q0, [x27], #32
	cmp x17, x26
	b.ne .LBB4332_18
	sub x25, x20, #4
	tbnz w16, #0, .LBB4332_22
	sub x26, x26, #1
	cmp x25, x9
	b.hs .LBB4332_15
	b .LBB4332_24
	.p2align	5, , 16
.LBB4332_21:
	mov x25, xzr
	mov w27, #4
	b .LBB4332_23
	.p2align	5, , 16
.LBB4332_22:
	mov x27, x20
.LBB4332_23:
	add x28, x23, x9, lsl #2
	lsl x25, x25, #2
	ldr q0, [x23, x25]
	ldr q1, [x28, x25]
	fadd v2.4s, v0.4s, v1.4s
	fsub v0.4s, v0.4s, v1.4s
	str q2, [x23, x25]
	str q0, [x28, x25]
	mov x25, x27
	cmp x27, x9
	b.hs .LBB4332_15
.LBB4332_24:
	cmp x14, #1
	b.ls .LBB4332_27
	mul x27, x12, x13
	add x28, x0, x26, lsl #4
	add x30, x3, x26, lsl #4
	add x28, x28, x27
	add x30, x30, x27
	add x28, x28, #16
	cmp x28, x30
	b.hs .LBB4332_29
	add x28, x18, x26, lsl #4
	add x26, x2, x26, lsl #4
	add x26, x26, x27
	add x28, x28, x27
	add x26, x26, #16
	cmp x26, x28
	b.hs .LBB4332_29
.LBB4332_27:
	mov x26, x25
	.p2align	5, , 16
.LBB4332_28:
	ldr s0, [x23, x26, lsl #2]
	ldr s1, [x22, x26, lsl #2]
	fadd s2, s0, s1
	fsub s0, s0, s1
	str s2, [x23, x26, lsl #2]
	str s0, [x22, x26, lsl #2]
	add x26, x26, #1
	cmp x26, x9
	b.lo .LBB4332_28
	b .LBB4332_15
.LBB4332_29:
	cmp x14, #8
	b.hs .LBB4332_31
	mov x28, xzr
	b .LBB4332_35
.LBB4332_31:
	add x26, x23, x25, lsl #2
	add x27, x19, x25, lsl #2
	and x28, x14, #0xfffffffffffffff8
	.p2align	5, , 16
.LBB4332_32:
	ldp q0, q1, [x26]
	ldp q2, q3, [x27, #-16]
	subs x28, x28, #8
	fadd v5.4s, v1.4s, v3.4s
	fsub v1.4s, v1.4s, v3.4s
	fadd v4.4s, v0.4s, v2.4s
	fsub v0.4s, v0.4s, v2.4s
	stp q4, q5, [x26], #32
	stp q0, q1, [x27, #-16]
	add x27, x27, #32
	b.ne .LBB4332_32
	cmp x14, x5
	b.eq .LBB4332_15
	and x28, x14, #0xfffffffffffffff8
	cbz x4, .LBB4332_38
.LBB4332_35:
	add x26, x25, x7
	add x25, x28, x25
	sub x27, x21, x28
	lsl x25, x25, #2
	.p2align	5, , 16
.LBB4332_36:
	ldr d0, [x23, x25]
	ldr d1, [x22, x25]
	subs x27, x27, #2
	fadd v2.2s, v0.2s, v1.2s
	fsub v0.2s, v0.2s, v1.2s
	str d2, [x23, x25]
	str d0, [x22, x25]
	add x25, x25, #8
	b.ne .LBB4332_36
	cbnz x6, .LBB4332_28
	b .LBB4332_15
.LBB4332_38:
	add x26, x25, x5
	b .LBB4332_28
.LBB4332_39:
	str x1, [sp, #8]
	adrp x9, :got:<usize as core::fmt::Display>::fmt
	add x8, sp, #8
	adrp x0, .Lanon.8f2aa15b57af37b749e6b281cbc37a85.11748
	add x0, x0, :lo12:.Lanon.8f2aa15b57af37b749e6b281cbc37a85.11748
	ldr x9, [x9, :got_lo12:<usize as core::fmt::Display>::fmt]
	adrp x2, .Lanon.8f2aa15b57af37b749e6b281cbc37a85.11750
	add x2, x2, :lo12:.Lanon.8f2aa15b57af37b749e6b281cbc37a85.11750
	add x1, sp, #16
	stp x8, x9, [sp, #16]
	bl core::panicking::panic_fmt
.LBB4332_40:
	mov x19, x1
	mov x20, x0
	bl std::sync::once_lock::OnceLock<T>::initialize
	mov x0, x20
	mov x1, x19
	adrp x8, ecaz::quant::simd::backend::BACKEND+4
	ldrb w8, [x8, :lo12:ecaz::quant::simd::backend::BACKEND+4]
	tbnz w8, #0, .LBB4332_3
.LBB4332_41:
	cmp x1, #2
	b.lo .LBB4332_56
	add x8, x0, #16
	mov w9, #1
	b .LBB4332_44
	.p2align	5, , 16
.LBB4332_43:
	mov x9, x10
	cmp x10, x1
	b.hs .LBB4332_56
.LBB4332_44:
	neg x11, x9, lsl #1
	lsl x10, x9, #1
	and x11, x1, x11
	cmp x10, x11
	b.hi .LBB4332_43
	and x12, x9, #0x6
	add x13, x8, x9, lsl #2
	lsl x14, x9, #3
	add x15, x0, x9, lsl #2
	mov x16, x0
	add x17, x0, #16
	b .LBB4332_47
	.p2align	5, , 16
.LBB4332_46:
	add x16, x16, x14
	sub x11, x11, x10
	add x13, x13, x14
	add x17, x17, x14
	add x15, x15, x14
	cmp x10, x11
	b.hi .LBB4332_43
.LBB4332_47:
	cmp x9, #1
	b.hi .LBB4332_50
	mov x18, xzr
	.p2align	5, , 16
.LBB4332_49:
	ldr s0, [x16, x18, lsl #2]
	ldr s1, [x15, x18, lsl #2]
	fadd s2, s0, s1
	fsub s0, s0, s1
	str s2, [x16, x18, lsl #2]
	str s0, [x15, x18, lsl #2]
	add x18, x18, #1
	cmp x9, x18
	b.ne .LBB4332_49
	b .LBB4332_46
	.p2align	5, , 16
.LBB4332_50:
	cmp x9, #8
	b.hs .LBB4332_54
	add x18, x16, x9, lsl #2
	ldr d0, [x16]
	ldr d1, [x18]
	fadd v2.2s, v0.2s, v1.2s
	fsub v0.2s, v0.2s, v1.2s
	str d2, [x16]
	str d0, [x18]
	cmp x12, #2
	b.eq .LBB4332_46
	ldr d0, [x16, #8]
	ldr d1, [x18, #8]
	fadd v2.2s, v0.2s, v1.2s
	fsub v0.2s, v0.2s, v1.2s
	str d2, [x16, #8]
	str d0, [x18, #8]
	cmp x12, #4
	b.eq .LBB4332_46
	ldr d0, [x16, #16]
	ldr d1, [x18, #16]
	fadd v2.2s, v0.2s, v1.2s
	fsub v0.2s, v0.2s, v1.2s
	str d2, [x16, #16]
	str d0, [x18, #16]
	b .LBB4332_46
	.p2align	5, , 16
.LBB4332_54:
	and x3, x9, #0x7ffffffffffffff8
	mov x18, x17
	mov x2, x13
	.p2align	5, , 16
.LBB4332_55:
	ldp q0, q1, [x18, #-16]
	ldp q2, q3, [x2, #-16]
	subs x3, x3, #8
	fadd v5.4s, v1.4s, v3.4s
	fsub v1.4s, v1.4s, v3.4s
	fadd v4.4s, v0.4s, v2.4s
	fsub v0.4s, v0.4s, v2.4s
	stp q4, q5, [x18, #-16]
	stp q0, q1, [x2, #-16]
	add x2, x2, #32
	add x18, x18, #32
	b.ne .LBB4332_55
	b .LBB4332_46
.LBB4332_56:
	.cfi_def_cfa wsp, 128
	ldp x20, x19, [sp, #112]
	ldp x22, x21, [sp, #96]
	ldp x24, x23, [sp, #80]
	ldp x26, x25, [sp, #64]
	ldp x28, x27, [sp, #48]
	ldp x29, x30, [sp, #32]
	add sp, sp, #128
	.cfi_def_cfa_offset 0
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore w23
	.cfi_restore w24
	.cfi_restore w25
	.cfi_restore w26
	.cfi_restore w27
	.cfi_restore w28
	.cfi_restore w30
	.cfi_restore w29
	ret
