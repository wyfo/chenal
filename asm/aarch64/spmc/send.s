spmc_send:
	stp x29, x30, [sp, #-32]!
	stp x20, x19, [sp, #16]
	mov x29, sp
	ldr x19, [x0]
	ldr x8, [x19, #128]
	mov w9, w8
	cmp x9, x8, lsr #32
	b.eq .LBB6_8
	add x9, x19, #568
	ldar x9, [x9]
	cbnz x9, .LBB6_8
	ldr x9, [x19, #560]
	ldr x10, [x19, #544]
	dmb ish
	and x9, x9, x8
	str x1, [x10, x9, lsl #3]
	ldr x10, [x19, #552]
	sub x10, x10, #1
	cmp x9, x10
	b.ne .LBB6_4
	ldr w9, [x19, #560]
	and x10, x8, #0xffffffff00000000
	orr w9, w8, w9
	add w9, w9, #1
	orr x9, x9, x10
	b .LBB6_5
.LBB6_4:
	add x9, x8, #1
.LBB6_5:
	add x10, x19, #128
	stlr x9, [x10]
	add x9, x19, #568
	ldar x9, [x9]
	cbnz x9, .LBB6_13
	add x0, x19, #424
	ldar x8, [x0]
	tbnz w8, #0, .LBB6_11
	mov x0, xzr
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
.LBB6_8:
	mov x20, x1
	ldr x1, [x2]
	add x0, x19, #128
	mov x2, x8
	bl chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold
	cmp x0, #2
	b.eq .LBB6_12
	mov x8, x1
	mov x1, x20
	cbz x0, .LBB6_2
.LBB6_10:
	mov w0, #1
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
	bl aiq::queue::Queue<T,S,SP>::is_empty_locked
	mov x0, xzr
.LBB6_12:
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
	add x0, x19, #128
	mov x1, x8
	bl <chenal::spmc::array::Array<C> as chenal::internal::Channel>::write_slot::handle_closed
	tbz w0, #0, .LBB6_6
	b .LBB6_10
