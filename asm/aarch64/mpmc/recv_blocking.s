mpmc_recv_blocking:
	sub sp, sp, #64
	stp x29, x30, [sp, #48]
	add x29, sp, #48
	ldr x8, [x0]
	ldr x9, [x8, #256]
	ldr x10, [x8, #584]
	ldr x12, [x8, #568]
	and x11, x10, x9
	mov x2, x9
	add x10, x12, x11, lsl #4
	add x12, x10, #8
	ldar x12, [x12]
	cmp x12, x9
	b.ne .LBB12_3
	ldr x10, [x10]
	dmb ishld
	ldr x12, [x8, #576]
	sub x12, x12, #1
	cmp x11, x12
	b.ne .LBB12_5
	ldr x11, [x8, #584]
	orr x11, x9, x11
	add x11, x11, #1
	add x12, x8, #256
	casal x2, x11, [x12]
	cmp x2, x9
	b.eq .LBB12_6
.LBB12_3:
	mov w9, #51712
	mov x0, sp
	add x1, x8, #128
	movk w9, #15258, lsl #16
	add x3, sp, #16
	str w9, [sp, #40]
	bl chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold
	ldrb w8, [sp]
	cmp w8, #1
	b.ne .LBB12_7
	mov w0, #1
	ldp x29, x30, [sp, #48]
	add sp, sp, #64
	ret
.LBB12_5:
	add x11, x9, #1
	add x12, x8, #256
	casal x2, x11, [x12]
	cmp x2, x9
	b.ne .LBB12_3
.LBB12_6:
	str x10, [sp, #8]
.LBB12_7:
	ldr x1, [sp, #8]
	mov x0, xzr
	ldp x29, x30, [sp, #48]
	add sp, sp, #64
	ret
