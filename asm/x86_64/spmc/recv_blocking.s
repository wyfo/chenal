spmc_recv_blocking:
	push rbx
	sub rsp, 48
	mov rsi, qword ptr [rdi]
	mov rax, qword ptr [rsi + 256]
	mov rcx, rax
	shr rcx, 32
	cmp eax, ecx
	je .LBB12_9
	mov rcx, qword ptr [rsi + 544]
	and rcx, rax
	mov rdx, qword ptr [rsi + 528]
	#MEMBARRIER
	mov rbx, qword ptr [rdx + 8*rcx]
	#MEMBARRIER
	mov rdx, qword ptr [rsi + 536]
	dec rdx
	cmp rcx, rdx
	jne .LBB12_3
	mov edx, dword ptr [rsi + 544]
	or edx, eax
	inc edx
	movabs rcx, -4294967296
	and rcx, rax
	or rcx, rdx
	lock cmpxchg	qword ptr [rsi + 256], rcx
	je .LBB12_5
.LBB12_9:
	mov dword ptr [rsp + 40], 1000000000
	sub rsi, -128
	mov rdi, rsp
	lea rcx, [rsp + 16]
	mov rdx, rax
	call chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold
	cmp byte ptr [rsp], 1
	jne .LBB12_8
	mov eax, 1
	add rsp, 48
	pop rbx
	ret
.LBB12_3:
	lea rcx, [rax + 1]
	lock cmpxchg	qword ptr [rsi + 256], rcx
	jne .LBB12_9
.LBB12_5:
	mov rax, qword ptr [rsi + 512]
	cmp rax, 1
	jbe .LBB12_6
.LBB12_7:
	mov qword ptr [rsp + 8], rbx
.LBB12_8:
	mov rdx, qword ptr [rsp + 8]
	xor eax, eax
	add rsp, 48
	pop rbx
	ret
.LBB12_6:
	add rsi, 480
	mov rdi, rsi
	mov rsi, rax
	call qword ptr [rip + spmc_waker::SpmcWaker<_,_>::wake_unsync_cold@GOTPCREL]
	jmp .LBB12_7
