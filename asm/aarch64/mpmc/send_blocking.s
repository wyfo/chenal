mpmc_send_blocking:
	sub sp, sp, #80
	stp x29, x30, [sp, #48]
	stp x20, x19, [sp, #64]
	add x29, sp, #48
	ldr x19, [x0]
	ldr x9, [x19, #128]
	mov w8, w9
	mov x2, x9
	cmp x8, x9, lsr #32
	b.eq .LBB18_5
	ldr x10, [x19, #584]
	ldr x11, [x19, #576]
	and x10, x10, x9
	sub x11, x11, #1
	cmp x10, x11
	b.hs .LBB18_5
	add x11, x9, #1
	add x12, x19, #128
	cas x2, x11, [x12]
	cmp x2, x9
	b.ne .LBB18_5
	ldr x9, [x19, #568]
	add x9, x9, x10, lsl #4
	stp x9, x8, [sp]
	dmb ishld
	str x1, [x9], #8
	add x10, x19, #440
	stlr x8, [x9]
	ldar x9, [x10]
	tbnz w9, #0, .LBB18_7
	mov x0, xzr
	ldp x20, x19, [sp, #64]
	ldp x29, x30, [sp, #48]
	add sp, sp, #80
	ret
.LBB18_5:
	mov w8, #51712
	mov x20, x1
	mov x0, sp
	movk w8, #15258, lsl #16
	add x1, x19, #128
	add x3, sp, #16
	str w8, [sp, #40]
	bl chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold
	ldr x9, [sp]
	cbz x9, .LBB18_8
	ldr x8, [sp, #8]
	mov x1, x20
	dmb ishld
	str x20, [x9], #8
	add x10, x19, #440
	stlr x8, [x9]
	ldar x9, [x10]
	tbz w9, #0, .LBB18_4
	add x0, x19, #128
	mov x19, x1
	mov x1, x8
	bl <chenal::mpmc::array::Array<C,U> as chenal::internal::Channel>::write_slot::notify_receivers
	mov x1, x19
	mov x0, xzr
	ldp x20, x19, [sp, #64]
	ldp x29, x30, [sp, #48]
	add sp, sp, #80
	ret
	mov w0, #1
	mov x1, x20
	ldp x20, x19, [sp, #64]
	ldp x29, x30, [sp, #48]
	add sp, sp, #80
	ret
