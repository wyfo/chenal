spmc_recv_blocking:
	push rbx
	sub rsp, 48
	mov rbx, qword ptr [rdi]
	mov rax, qword ptr [rbx + 256]
	mov rcx, rax
	shr rcx, 32
	cmp eax, ecx
	je .LBB13_9
	mov rdx, qword ptr [rbx + 544]
	and rdx, rax
	mov rcx, qword ptr [rbx + 528]
	mov rcx, qword ptr [rcx + 8*rdx]
	#MEMBARRIER
	mov rsi, qword ptr [rbx + 536]
	dec rsi
	cmp rdx, rsi
	jne .LBB13_3
	mov esi, dword ptr [rbx + 544]
	or esi, eax
	inc esi
	movabs rdx, -4294967296
	and rdx, rax
	or rdx, rsi
	lock cmpxchg	qword ptr [rbx + 256], rdx
	je .LBB13_5
.LBB13_9:
	mov dword ptr [rsp + 40], 1000000000
	lea rsi, [rbx + 128]
	mov rdi, rsp
	lea rcx, [rsp + 16]
	mov rdx, rax
	call chenal::channel::Chan<T,Ch,SP>::acquire_slot_blocking_cold
	cmp byte ptr [rsp], 1
	jne .LBB13_6
	mov eax, 1
	add rsp, 48
	pop rbx
	ret
.LBB13_3:
	lea rdx, [rax + 1]
	lock cmpxchg	qword ptr [rbx + 256], rdx
	jne .LBB13_9
.LBB13_5:
	mov qword ptr [rsp + 8], rcx
.LBB13_6:
	mov rdx, qword ptr [rsp + 8]
	mov rsi, qword ptr [rbx + 512]
	cmp rsi, 1
	jbe .LBB13_7
	xor eax, eax
	add rsp, 48
	pop rbx
	ret
.LBB13_7:
	add rbx, 480
	mov rdi, rbx
	mov rbx, rdx
	call qword ptr [rip + spmc_waker::SpmcWaker<_,_>::wake_unsync_cold@GOTPCREL]
	mov rdx, rbx
	xor eax, eax
	add rsp, 48
	pop rbx
	ret
