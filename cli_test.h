#pragma once


#include "packetfactory.h"
#include <array>

namespace RoseCommon {
namespace Packet {

class CliTest : public CRosePacket {
    public:
        CliTest();
        CliTest(CRoseReader reader);
        CliTest(CliTest&&) = default;
        CliTest& operator=(CliTest&&) = default;
        ~CliTest() = default;
        
        static constexpr size_t size();
        
        
        struct A : public ISerialize {
            A();
            A(int);
            A(const A&) = default;
            A(A&&) = default;
            A& operator=(const A&) = default;
            A& operator=(A&&) = default;
            virtual ~A() = default;
            
            static constexpr size_t size();
            
            operator int() const { return a; }
            bool isValid() const { return is_valid; }
            
            virtual bool read(CRoseReader&) override;
            virtual bool write(CRoseBasePolicy&) const override;
            
            private:
                int a;
                bool is_valid;
        };
        
        struct B : public ISerialize {
            B();
            B(int);
            B(const B&) = default;
            B(B&&) = default;
            B& operator=(const B&) = default;
            B& operator=(B&&) = default;
            virtual ~B() = default;
            
            static constexpr size_t size();
            
            operator int() const { return b; }
            bool isValid() const { return is_valid; }
            
            virtual bool read(CRoseReader&) override;
            virtual bool write(CRoseBasePolicy&) const override;
            
            private:
                int b;
                bool is_valid;
        };
        
        struct C : public ISerialize {
            C();
            C(int);
            C(const C&) = default;
            C(C&&) = default;
            C& operator=(const C&) = default;
            C& operator=(C&&) = default;
            virtual ~C() = default;
            
            static constexpr size_t size();
            
            operator int() const { return c; }
            bool isValid() const { return is_valid; }
            
            virtual bool read(CRoseReader&) override;
            virtual bool write(CRoseBasePolicy&) const override;
            
            private:
                int c;
                bool is_valid;
        };
        
        
        void set_test(const std::array<int, 42>&);
        void set_test(const int&, size_t index);
        const std::array<int, 42>& get_test() const;
        const int& get_test(size_t index) const;
        
        
        static CliTest create(const std::array<int, 42>&);
        static CliTest create(const uint8_t*);
        static std::unique_ptr<CliTest> allocate(const uint8_t*);
    
    protected:
        virtual void pack(CRoseBasePolicy&) const override;
    
    private:
        std::array<int,42> test;
};

}
}
