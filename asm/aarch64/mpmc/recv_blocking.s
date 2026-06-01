mpmc_recv_blocking:
	sub sp, sp, #80
	stp x29, x30, [sp, #48]
	str x19, [sp, #64]
	add x29, sp, #48
	ldr x8, [x0]
	ldr x9, [x8, #256]
	ldr x10, [x8, #584]
	ldr x11, [x8, #568]
	and x10, x10, x9
	mov x2, x9
	add x11, x11, x10, lsl #4
	add x12, x11, #8
	ldar x12, [x12]
	cmp x12, x9
	b.ne .LBB14_3
	ldr x19, [x11]
	dmb ishld
	ldr x11, [x8, #576]
	sub x11, x11, #1
	cmp x10, x11
	b.ne .LBB14_5
	ldr x10, [x8, #584]
	orr x10, x9, x10
	add x10, x10, #1
	add x11, x8, #256
	casal x2, x10, [x11]
	cmp x2, x9
	b.eq .LBB14_6
.LBB14_3:
	mov w9, #51712
	mov x0, sp
	add x1, x8, #128
	movk w9, #15258, lsl #16
	add x3, sp, #16
	str w9, [sp, #40]
	bl chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold
	ldrb w8, [sp]
	cmp w8, #1
	b.ne .LBB14_8
	mov w0, #1
	ldp x29, x30, [sp, #48]
	ldr x19, [sp, #64]
	add sp, sp, #80
	ret
.LBB14_5:
	add x10, x9, #1
	add x11, x8, #256
	casal x2, x10, [x11]
	cmp x2, x9
	b.ne .LBB14_3
.LBB14_6:
	add x0, x8, #384
	ldar x8, [x0]
	tbnz w8, #0, .LBB14_9
.LBB14_7:
	str x19, [sp, #8]
.LBB14_8:
	ldr x1, [sp, #8]
	mov x0, xzr
	ldp x29, x30, [sp, #48]
	ldr x19, [sp, #64]
	add sp, sp, #80
	ret
	bl aiq::queue::Queue<T,S,SP>::is_empty_locked
	b .LBB14_7
