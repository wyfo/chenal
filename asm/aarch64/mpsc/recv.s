mpsc_recv:
	sub sp, sp, #64
	stp x29, x30, [sp, #32]
	str x19, [sp, #48]
	add x29, sp, #32
	ldr x19, [x0]
	ldr x4, [x19, #256]
	ldr x8, [x19, #560]
	ldr x9, [x19, #544]
	and x8, x8, x4
	add x3, x9, x8, lsl #4
	add x8, x3, #8
	ldar x8, [x8]
	cmp x8, x4
	b.ne .LBB7_3
.LBB7_1:
	ldr x8, [x19, #560]
	ldr x9, [x19, #552]
	add x0, x19, #384
	ldr x1, [x3]
	and x10, x8, x4
	sub x9, x9, #1
	orr x8, x4, x8
	cmp x10, x9
	add x8, x8, #1
	add x9, x19, #256
	csinc x8, x8, x4, eq
	stlr x8, [x9]
	ldar x8, [x0]
	tbnz w8, #0, .LBB7_5
	mov x0, xzr
	ldp x29, x30, [sp, #32]
	ldr x19, [sp, #48]
	add sp, sp, #64
	ret
.LBB7_3:
	ldr x2, [x1]
	add x0, sp, #8
	add x1, x19, #128
	bl chenal::channel::Chan<T,Ch,SP>::poll_acquire_slot_cold
	ldr w8, [sp, #8]
	tbz w8, #0, .LBB7_6
	mov w0, #2
	ldp x29, x30, [sp, #32]
	ldr x19, [sp, #48]
	add sp, sp, #64
	ret
	mov x19, x1
	bl aiq::queue::Queue<T,S,SP>::is_empty_locked
	mov x1, x19
	mov x0, xzr
	ldp x29, x30, [sp, #32]
	ldr x19, [sp, #48]
	add sp, sp, #64
	ret
	ldr x3, [sp, #16]
	cbz x3, .LBB7_8
	ldr x4, [sp, #24]
	b .LBB7_1
	mov w0, #1
	ldp x29, x30, [sp, #32]
	ldr x19, [sp, #48]
	add sp, sp, #64
	ret
