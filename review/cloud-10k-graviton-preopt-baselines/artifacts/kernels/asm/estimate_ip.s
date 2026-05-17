.section .text.ecaz::quant::rabitq::RaBitQQuantizer::estimate_ip,"ax",@progbits
	.globl	ecaz::quant::rabitq::RaBitQQuantizer::estimate_ip
	.p2align	4
.type	ecaz::quant::rabitq::RaBitQQuantizer::estimate_ip,@function
ecaz::quant::rabitq::RaBitQQuantizer::estimate_ip:
	.cfi_startproc
	ldr s0, [x1, #32]
	mov x5, x3
	mov x4, x2
	ldp x0, x8, [x1, #8]
	ldr x2, [x1, #24]
	ldrb w3, [x1, #36]
	mov x1, x8
	b ecaz::quant::rabitq::estimate_ip_impl
