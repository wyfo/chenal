mpsc_recv_blocking:
	sub sp, sp, #80
	stp x29, x30, [sp, #48]
	str x19, [sp, #64]
	add x29, sp, #48
	ldr x19, [x0]
	ldr x3, [x19, #256]
	ldr x8, [x19, #560]
	ldr x9, [x19, #544]
	and x8, x8, x3
	add x2, x9, x8, lsl #4
	add x8, x2, #8
	ldar x8, [x8]
	cmp x8, x3
	b.ne .LBB7_3
.LBB7_1:
	ldr x8, [x19, #560]
	ldr x9, [x19, #552]
	add x0, x19, #384
	ldr x1, [x2]
	and x10, x8, x3
	sub x9, x9, #1
	orr x8, x3, x8
	cmp x10, x9
	add x8, x8, #1
	add x9, x19, #256
	csinc x8, x8, x3, eq
	stlr x8, [x9]
	ldar x8, [x0]
	tbnz w8, #0, .LBB7_5
	mov x0, xzr
	ldp x29, x30, [sp, #48]
	ldr x19, [sp, #64]
	add sp, sp, #80
	ret
.LBB7_3:
	mov w8, #51712
	mov x0, sp
	add x1, x19, #128
	movk w8, #15258, lsl #16
	add x4, sp, #16
	str w8, [sp, #40]
	bl chenal::channel::Chan<T,Ch,SP>::acquire_slot_blocking_cold
	ldr x2, [sp]
	cbz x2, .LBB7_6
	ldr x3, [sp, #8]
	b .LBB7_1
	mov x19, x1
	bl aiq::queue::Queue<T,S,SP>::is_empty_locked
	mov x1, x19
	mov x0, xzr
	ldp x29, x30, [sp, #48]
	ldr x19, [sp, #64]
	add sp, sp, #80
	ret
	mov w0, #1
	ldp x29, x30, [sp, #48]
	ldr x19, [sp, #64]
	add sp, sp, #80
	ret
