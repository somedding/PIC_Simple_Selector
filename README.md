# PhotoSelector

사진 선택 및 관리 프로그램

## 실행 방법

### 1. 배포 파일 실행
- **Windows**: `PhotoSelector.exe` 실행
- **macOS**: `PhotoSelector` 실행 (터미널에서 `./PhotoSelector`)
- **Linux**: `PhotoSelector` 실행 (터미널에서 `./PhotoSelector`)

### 2. 개발자용 실행 방법
- Rust 설치 ([https://rustup.rs/](https://rustup.rs/))
- 터미널에서 다음 명령어 실행:
```bash
git clone [repository-url]
cd PhotoSelector
cargo run --release
```

## 개발 정보

### 1. 사용 언어
- Rust 2021 Edition

### 2. 주요 라이브러리
- `iced` (v0.10): GUI 프레임워크
- `image` (v0.24): 이미지 처리
- `walkdir` (v2.4): 파일 시스템 탐색
- `kamadak-exif` (v0.5): EXIF 데이터 처리
- `rayon` (v1.8): 병렬 처리
- `rfd` (v0.11): 파일 대화상자

## 주요 기능

### 1. 사진 탐색
- 프로그램 실행 시 자동으로 폴더 선택 창이 열립니다
- "Select Folder" 버튼으로 언제든지 다른 폴더 선택 가능
- JPG/JPEG 파일 자동 인식
- RAW 파일(DNG, CR2, NEF, ARW 등) 자동 연결

### 2. 사진 보기
- 큰 화면으로 사진 확인 가능
- EXIF 정보 표시
  * 촬영 일시
  * 카메라 모델
  * 초점 거리
  * ISO
  * 조리개 값
  * 셔터 스피드
- 현재 사진 번호/전체 사진 수 표시

### 3. 키보드 단축키
- `←` (왼쪽 화살표): 이전 사진
- `→` (오른쪽 화살표): 다음 사진
- `S`: 현재 사진 선택 후 다음 사진으로 이동
- `D`: 현재 사진 삭제

### 4. 사진 관리
- 선택/삭제 기능
- RAW+JPG 연동 삭제 지원
  * JPG 파일 삭제 시 연결된 RAW 파일도 함께 삭제
- 배치(Batch) 처리 지원
  * 10장씩 자동 로드
  * 메모리 사용 최적화

### 5. 성능 최적화
- 대용량 사진의 자동 리사이징
- 세로 사진 자동 비율 조정
- 메모리 캐시 관리
- 비동기 이미지 로딩

## 사용 방법
1. 프로그램 실행
2. 폴더 선택 창에서 사진이 있는 폴더 선택
3. 화살표 키나 버튼으로 사진 탐색
4. S 키나 "Select" 버튼으로 보관할 사진 선택
5. D 키나 "Delete" 버튼으로 불필요한 사진 삭제

## 주의사항
- ⚠️ 삭제된 파일은 복구할 수 없습니다
- ⚠️ RAW+JPG 연동 삭제 기능 사용 시 주의 필요

## 시스템 요구사항
- Windows/macOS/Linux 지원
- 그래픽 사용자 인터페이스(GUI) 지원
- 충분한 메모리 공간
- 저장 공간: 최소 100MB

## 빌드 요구사항
- Rust 1.70.0 이상
- Cargo (Rust 패키지 매니저)
- C++ 빌드 도구
  * Windows: Visual Studio Build Tools
  * macOS: Xcode Command Line Tools
  * Linux: GCC 또는 Clang

## 프로젝트 구조
```
PhotoSelector/
├── src/
│   └── main.rs          # 메인 소스 코드
├── Cargo.toml           # 프로젝트 설정 및 의존성
├── README.md            # 설명서
├── build_windows.bat    # Windows 배포 스크립트
├── build_mac.sh         # macOS 배포 스크립트
└── build_linux.sh       # Linux 배포 스크립트
```

## 알려진 문제
- 매우 큰 이미지 파일(100MB 이상)의 경우 로딩 시간이 길어질 수 있음
- 일부 특수한 형식의 RAW 파일은 인식되지 않을 수 있음

## 향후 계획
- [ ] 더 많은 RAW 파일 형식 지원
- [ ] 이미지 편집 기능 추가
- [ ] 성능 최적화
- [ ] 다국어 지원

## 기여하기
1. 이슈 제보: GitHub Issues를 통해 버그 리포트 또는 기능 제안
2. 풀 리퀘스트: 코드 기여는 언제나 환영합니다
3. 문서화: README나 코드 문서 개선에 참여 가능

## 문의 및 지원
- GitHub Issues: [repository-url]/issues
- 이메일: [이메일 주소]

## 라이선스
MIT

## 버전
1.0.0 