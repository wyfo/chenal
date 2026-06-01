mpmc_recv_blocking:
	sub sp, sp, #80
	stp x29, x30, [sp, #48]
	str x19, [sp, #64]
	add x29, sp, #48
	ldr x19, [x0]
	ldr x8, [x19, #256]
	ldr x9, [x19, #584]
	ldr x11, [x19, #568]
	and x10, x9, x8
	mov x2, x8
	add x9, x11, x10, lsl #4
	add x11, x9, #8
	ldar x11, [x11]
	cmp x11, x8
	b.ne .LBB12_3
	ldr x9, [x9]
	dmb ishld
	ldr x11, [x19, #576]
	sub x11, x11, #1
	cmp x10, x11
	b.ne .LBB12_5
	ldr x10, [x19, #584]
	orr x10, x8, x10
	add x10, x10, #1
	add x11, x19, #256
	casal x2, x10, [x11]
	cmp x2, x8
	b.eq .LBB12_6
.LBB12_3:
	mov w8, #51712
	mov x0, sp
	add x1, x19, #128
	movk w8, #15258, lsl #16
	add x3, sp, #16
	str w8, [sp, #40]
	bl chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold
	ldrb w8, [sp]
	cmp w8, #1
	b.ne .LBB12_7
	mov w0, #1
	ldp x29, x30, [sp, #48]
	ldr x19, [sp, #64]
	add sp, sp, #80
	ret
.LBB12_5:
	add x10, x8, #1
	add x11, x19, #256
	casal x2, x10, [x11]
	cmp x2, x8
	b.ne .LBB12_3
.LBB12_6:
	str x9, [sp, #8]
.LBB12_7:
	add x0, x19, #384
	ldr x1, [sp, #8]
	ldar x8, [x0]
	tbnz w8, #0, .LBB12_9
	mov x0, xzr
	ldp x29, x30, [sp, #48]
	ldr x19, [sp, #64]
	add sp, sp, #80
	ret
	mov x19, x1
	bl aiq::queue::Queue<T,S,SP>::is_empty_locked
	mov x1, x19
	mov x0, xzr
	ldp x29, x30, [sp, #48]
	ldr x19, [sp, #64]
	add sp, sp, #80
	ret
