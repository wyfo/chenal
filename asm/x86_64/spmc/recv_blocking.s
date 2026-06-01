spmc_recv_blocking:
	sub rsp, 56
	mov rsi, qword ptr [rdi]
	mov rax, qword ptr [rsi + 256]
	mov rcx, rax
	shr rcx, 32
	cmp eax, ecx
	je .LBB12_7
	mov rdx, qword ptr [rsi + 544]
	and rdx, rax
	mov rcx, qword ptr [rsi + 528]
	mov rcx, qword ptr [rcx + 8*rdx]
	#MEMBARRIER
	mov rdi, qword ptr [rsi + 536]
	dec rdi
	cmp rdx, rdi
	jne .LBB12_3
	mov edi, dword ptr [rsi + 544]
	or edi, eax
	inc edi
	movabs rdx, -4294967296
	and rdx, rax
	or rdx, rdi
	lock cmpxchg	qword ptr [rsi + 256], rdx
	je .LBB12_5
.LBB12_7:
	mov dword ptr [rsp + 48], 1000000000
	sub rsi, -128
	lea rdi, [rsp + 8]
	lea rcx, [rsp + 24]
	mov rdx, rax
	call chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold
	cmp byte ptr [rsp + 8], 1
	jne .LBB12_6
	mov eax, 1
	add rsp, 56
	ret
.LBB12_3:
	lea rdx, [rax + 1]
	lock cmpxchg	qword ptr [rsi + 256], rdx
	jne .LBB12_7
.LBB12_5:
	mov qword ptr [rsp + 16], rcx
.LBB12_6:
	mov rdx, qword ptr [rsp + 16]
	xor eax, eax
	add rsp, 56
	ret
