import os
import platform
import subprocess
import shutil

def build_release():
    subprocess.run(['cargo', 'build', '--release'])

def create_package():
    # 배포 폴더 생성
    os.makedirs('PhotoSelector', exist_ok=True)
    
    # 실행 파일 복사
    if platform.system() == 'Windows':
        src = 'target/release/photo-selector.exe'
        dst = 'PhotoSelector/PhotoSelector.exe'
    else:
        src = 'target/release/photo-selector'
        dst = 'PhotoSelector/PhotoSelector'
        
    shutil.copy2(src, dst)
    
    # README 생성
    with open('PhotoSelector/README.txt', 'w', encoding='utf-8') as f:
        f.write('''PhotoSelector

사용 방법:
1. PhotoSelector 실행
2. "Select Folder" 버튼을 클릭하여 사진이 있는 폴더 선택
3. 키보드 단축키:
   - 좌/우 화살표: 이전/다음 사진
   - S: 현재 사진 선택
   - D: 현재 사진 삭제
''')
    
    # 압축
    if platform.system() == 'Windows':
        shutil.make_archive('PhotoSelector-Windows', 'zip', 'PhotoSelector')
    elif platform.system() == 'Darwin':
        shutil.make_archive('PhotoSelector-Mac', 'zip', 'PhotoSelector')
    else:
        shutil.make_archive('PhotoSelector-Linux', 'gztar', 'PhotoSelector')

if __name__ == '__main__':
    build_release()
    create_package() 