mpmc_recv_blocking:
	push rbx
	sub rsp, 48
	mov rsi, qword ptr [rdi]
	mov rax, qword ptr [rsi + 256]
	mov rcx, qword ptr [rsi + 560]
	and rcx, rax
	mov rdx, qword ptr [rsi + 544]
	mov rdi, rcx
	shl rdi, 4
	mov r8, qword ptr [rdx + rdi + 8]
	cmp r8, rax
	jne .LBB14_9
	add rdx, rdi
	mov rbx, qword ptr [rdx]
	#MEMBARRIER
	mov rdx, qword ptr [rsi + 552]
	dec rdx
	cmp rcx, rdx
	jne .LBB14_3
	mov rcx, qword ptr [rsi + 560]
	or rcx, rax
	inc rcx
	lock cmpxchg	qword ptr [rsi + 256], rcx
	je .LBB14_5
.LBB14_9:
	mov dword ptr [rsp + 40], 1000000000
	sub rsi, -128
	mov rdi, rsp
	lea rcx, [rsp + 16]
	mov rdx, rax
	call chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold
	cmp byte ptr [rsp], 1
	jne .LBB14_8
	mov eax, 1
	add rsp, 48
	pop rbx
	ret
.LBB14_3:
	lea rcx, [rax + 1]
	lock cmpxchg	qword ptr [rsi + 256], rcx
	jne .LBB14_9
.LBB14_5:
	mov rax, qword ptr [rsi + 384]
	test al, 1
	jne .LBB14_6
.LBB14_7:
	mov qword ptr [rsp + 8], rbx
.LBB14_8:
	mov rdx, qword ptr [rsp + 8]
	xor eax, eax
	add rsp, 48
	pop rbx
	ret
.LBB14_6:
	add rsi, 384
	mov rdi, rsi
	call qword ptr [rip + aiq::queue::Queue<T,S,SP>::is_empty_locked@GOTPCREL]
	jmp .LBB14_7
