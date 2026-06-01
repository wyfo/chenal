spmc_send_blocking:
	sub sp, sp, #80
	stp x29, x30, [sp, #48]
	stp x20, x19, [sp, #64]
	add x29, sp, #48
	ldr x19, [x0]
	ldr x2, [x19, #128]
	mov w8, w2
	cmp x8, x2, lsr #32
	b.eq .LBB6_9
	add x8, x19, #568
	ldar x8, [x8]
	cbnz x8, .LBB6_9
	str x2, [sp, #8]
	strb wzr, [sp]
	ldr x8, [sp, #8]
	ldr x9, [x19, #560]
	ldr x10, [x19, #544]
	dmb ish
	and x9, x9, x8
	str x1, [x10, x9, lsl #3]
	ldr x10, [x19, #552]
	sub x10, x10, #1
	cmp x9, x10
	b.ne .LBB6_5
	ldr w9, [x19, #560]
	and x10, x8, #0xffffffff00000000
	orr w9, w8, w9
	add w9, w9, #1
	orr x9, x9, x10
	b .LBB6_6
.LBB6_5:
	add x9, x8, #1
.LBB6_6:
	add x10, x19, #128
	stlr x9, [x10]
	add x9, x19, #568
	ldar x9, [x9]
	cbnz x9, .LBB6_12
	add x0, x19, #424
	ldar x8, [x0]
	tbnz w8, #0, .LBB6_11
	mov x0, xzr
	ldp x20, x19, [sp, #64]
	ldp x29, x30, [sp, #48]
	add sp, sp, #80
	ret
.LBB6_9:
	mov w8, #51712
	mov x0, sp
	mov x20, x1
	movk w8, #15258, lsl #16
	add x1, x19, #128
	add x3, sp, #16
	str w8, [sp, #40]
	bl chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold
	ldrb w8, [sp]
	mov x1, x20
	tbz w8, #0, .LBB6_3
	mov w0, #1
	ldp x20, x19, [sp, #64]
	ldp x29, x30, [sp, #48]
	add sp, sp, #80
	ret
	bl aiq::queue::Queue<T,S,SP>::is_empty_locked
	mov x0, xzr
	ldp x20, x19, [sp, #64]
	ldp x29, x30, [sp, #48]
	add sp, sp, #80
	ret
	add x0, x19, #128
	mov x1, x8
	bl <chenal::spmc::array::Array<C,SP> as chenal::internal::Channel>::write_slot::handle_closed
	tbz w0, #0, .LBB6_7
	mov w0, #1
	ldp x20, x19, [sp, #64]
	ldp x29, x30, [sp, #48]
	add sp, sp, #80
	ret
